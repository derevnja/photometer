use iron::typemap::Key;
use std::io;
use std::process::Command;
use std::fs::File;
use std::io::stderr;
use std::io::Write;
use std::thread;
use std::convert::From;
use std::path::Path;

use authentication::User;
use std::sync::mpsc::{ Sender, channel, SendError };
use std::sync::{ Arc, Mutex };
use db::mailbox::DbMailbox;
use types::{ EmptyResult, CommonError };
use database::{ Databaseable };
use stuff::{ Stuff, StuffInstallable };

struct MailSender( Sender<Mail> );

const PHOTOMETER : &'static str = "Фотометр";

pub trait Mailer {
    fn send_mail( &mut self, user: &User, subject: &str, body: &str  ) -> EmptyResult;
    fn send_external_mail( &mut self, user: &User, subject: &str, body: &str ) -> EmptyResult;
    fn send_internal_mail( &mut self, user: &User, subject: &str, body: &str ) -> EmptyResult;
}

pub fn create( context: MailContext ) -> MailerBody {
    MailerBody {
        etalon_sender: Arc::new( Mutex::new( create_mail_thread( context ) ) )
    }
}

impl Mailer for Stuff {
    fn send_mail( &mut self, user: &User, subject: &str, body: &str ) -> EmptyResult {
        try!( self.send_internal_mail( user, subject, body ) );
        try!( self.send_external_mail( user, subject, body ) );
        Ok( () )
    }
    fn send_external_mail( &mut self, user: &User, subject: &str, body: &str ) -> EmptyResult {
        self.send_mail_external( user, subject, body )
    }
    fn send_internal_mail( &mut self, user: &User, subject: &str, body: &str ) -> EmptyResult {
        self.send_mail_internal( user, PHOTOMETER, subject, body )
    }
}

trait MailerPrivate {
    fn send_mail_external( &mut self, user: &User, subject: &str, body: &str ) -> EmptyResult;
    fn send_mail_internal( &mut self, user: &User, sender: &str, subject: &str, body: &str ) -> EmptyResult;
}

impl MailerPrivate for Stuff {
    fn send_mail_external( &mut self, user: &User, subject: &str, body: &str ) -> EmptyResult {
        // здесь реализовано ленивое создание посыльщика писем с кешированием
        // елси в этом контексте мы его уже создавали то просто используем
        // а елси не создавали то создаём копию для текущего потока с эталона и кэшируем его
        if self.extensions.contains::<MailSender>() == false {
            let tx = {
                let body = self.extensions.get::<MailerBody>().unwrap();
                let sender: &MailSender = &body.etalon_sender.lock().unwrap();
                let &MailSender( ref tx ) = sender;
                tx.clone()
            };
            //кэшируем
            self.extensions.insert::<MailSender>( MailSender( tx ) );
        }
        //отсылаем в поток посылки почты новое письмо
        let &MailSender( ref tx ) = self.extensions.get::<MailSender>().unwrap();
        try!( tx.send( Mail {
            to_addr: user.mail.clone(),
            to_name: user.name.clone(),
            //sender_name: sender.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
        } ) );
        Ok( () )
    }

    fn send_mail_internal( &mut self, user: &User, sender: &str, subject: &str, body: &str ) -> EmptyResult {
        // делаем запись в базе о новом оповещении
        let db = try!( self.get_current_db_conn() );
        try!( db.send_mail_to( user.id, sender, subject, body ) );
        Ok( () )
    }
}

impl From<SendError<Mail>> for CommonError {
    fn from(err: SendError<Mail>) -> CommonError {
        CommonError( format!( "error sending mail to mailer channel: {}", err ) )
    }
}

#[derive(Clone)]
pub struct MailerBody {
    etalon_sender: Arc<Mutex<MailSender>>
}

pub struct MailContext {
    smtp_addr: String,
    from_addr: String,
    tmp_mail_file: String,
    auth_info: String
}

// сделал pub потому что иначе компилятор не даёт его использовать в FromError
pub struct Mail {
    to_addr: String,
    to_name: String,
    //sender_name: String,
    subject: String,
    body: String
}

impl Key for MailerBody { type Value = MailerBody; }
impl Key for MailSender { type Value = MailSender; }

//curl --url "smtps://smtp.gmail.com:465" --ssl-reqd --mail-from "photometer.org.ru@gmail.com" --mail-rcpt "voidwalker@mail.ru" --upload-file ./mail.txt --user "photometer.org.ru@gmail.com:ajnjvtnhbxtcrbq" --insecure
impl MailContext {
    pub fn new( smtp_addr: &str, from_addr: &str, pass: &str, tmp_file_path: &str ) -> MailContext {
        MailContext {
            smtp_addr: smtp_addr.to_string(),
            from_addr: from_addr.to_string(),//format!( "{}", from_addr ),
            tmp_mail_file: tmp_file_path.to_string(),
            auth_info: format!( "{}:{}", from_addr, pass )
        }
    }
    fn send_mail( &self, mail: Mail ) {
        //создаём текстовый файл с письмом
        if let Err( e ) = self.make_mail_file( &mail ) {
            let _ = writeln!( &mut stderr(), "fail to create tmp mail file: {}", e );
            return;
        }
        //запускаем curl на передачу записанного письма
        let process = Command::new( "curl" )
            .arg( "--url" )
            .arg( &self.smtp_addr )
            .arg( "--ssl-reqd" )
            .arg( "--mail-from" )
            .arg( &self.from_addr )
            .arg( "--mail-rcpt" )
            .arg( &format!( "\"{}\"", mail.to_addr ) )
            .arg( "--upload-file" )
            .arg( &self.tmp_mail_file )
            .arg( "--user" )
            .arg( &self.auth_info )
            .arg( "--insecure" )
            .output();

        let process = match process {
            Ok( process ) => process,
            Err( e ) => panic!( "fail to create 'curl' process: {}", e )
        };
        if process.status.success() == false {
            let err_string = String::from_utf8_lossy( &process.stderr );
            let _ = writeln!( &mut stderr(), "fail to send mail: {}", err_string );
        }
        else {
            debug!( "mail to '{}' with subject='{}' successfully sended.", mail.to_addr, mail.subject );
        }
    }
    fn make_mail_file( &self, mail: &Mail ) -> io::Result<()> {
        let ref mut file = try!( File::create( &Path::new( &self.tmp_mail_file ) ) );
        try!( writeln!( file, "From: \"photometer\" <{}>", self.from_addr ) );
        try!( writeln!( file, "To: \"{}\" <{}>", mail.to_name, mail.to_addr ) );
        try!( writeln!( file, "Subject: {}", mail.subject ) );
        try!( writeln!( file, "" ) );
        try!( write!( file, "{}", &mail.body ) );
        Ok( () )
    }
}

impl StuffInstallable for MailerBody {
    fn install_to( &self, stuff: &mut Stuff ) {
        stuff.extensions.insert::<MailerBody>( self.clone() );
    }
}

fn create_mail_thread( context: MailContext ) -> MailSender {
    let (tx, rx) = channel();

    thread::spawn( move || {
        loop {
            match rx.recv() {
                Ok( mail ) => context.send_mail( mail ),
                // Если все те кто могли послать ушли то и мы закрываемся
                Err( _ ) => break
            }
        }
        info!( "mail send loop closed" );
    });

    MailSender( tx )
}
