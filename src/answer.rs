use rustc_serialize::{ Encodable };
use rustc_serialize::json;
use iron::modifier::Modifier;
use iron::prelude::*;
use iron::mime;
use iron::status;

use types::{ CommonResult, CommonError };

pub trait AnswerSendable {
    fn send_answer( &mut self, answer: &AnswerResult );
}

type ToStringPtr = Box<ToString>;

//TODO: после обновления до беты убрать static lifetime
struct AnswerBody<Body: Encodable + 'static> {
    body: Body
}

fn new_body<Body: Encodable + 'static>( body: Body ) -> AnswerBody<Body> {
    AnswerBody {
        body: body
    }
}

impl<Body: Encodable + 'static> ToString for AnswerBody<Body> {
    fn to_string( &self ) -> String {
        json::encode( &self.body ).unwrap()
    }
}

pub enum Answer {
    Good( ToStringPtr ),
    Bad( ToStringPtr )
}

pub type AnswerResult = CommonResult<Answer>;
pub struct AnswerResponse( pub AnswerResult );

impl Answer {
    pub fn good<Body: Encodable + 'static>(body: Body) -> Answer {
        Answer::Good( Box::new( new_body( body ) ) as ToStringPtr )
    }

    pub fn bad<Body: Encodable + 'static>(body: Body) -> Answer {
        Answer::Bad( Box::new( new_body( body ) ) as ToStringPtr )
    }
}

impl Modifier<Response> for AnswerResponse {
    #[inline]
    fn modify(self, res: &mut Response) {
        match self {
            AnswerResponse( Ok( ref answer ) ) => {
                //TODO: переделать на константу
                let mime: mime::Mime = "application/json;charset=utf8".parse().unwrap();
                mime.modify( res );

                match answer {
                    &Answer::Good( ref body ) => {
                        let answer_status = status::Ok;
                        answer_status.modify( res );
                        body.to_string().modify( res );
                    }

                    &Answer::Bad( ref body ) => {
                        let answer_status = status::BadRequest;
                        answer_status.modify( res );
                        body.to_string().modify( res );
                    }
                }
            }

            AnswerResponse( Err( CommonError( err ) ) ) => {
                let answer_status = status::InternalServerError;
                answer_status.modify( res );
                err.modify( res );
            }
        }
    }
}

impl From<CommonError> for AnswerResult {
    fn from( err: CommonError ) -> AnswerResult {
        Err( err )
    }
}

// impl ToJson for PhotoInfo {
//     fn to_json(&self) -> Json {
//         let mut d = BTreeMap::new();
//         d.add( "id", &self.id );
//         d.add( "type", &self.image_type );
//         d.add( "width", &self.width );
//         d.add( "height", &self.height );
//         d.add( "name", &self.name );
//         d.add( "iso", &self.iso );
//         d.add( "shutter", &self.shutter_speed );
//         d.add( "aperture", &self.aperture );
//         d.add( "focal_length", &self.focal_length );
//         d.add( "focal_length_35mm", &self.focal_length_35mm );
//         d.add( "camera_model", &self.camera_model );
//         Json::Object( d )
//     }
// }


// impl ToJson for MailInfo {
//     fn to_json( &self ) -> Json {
//         let mut d = BTreeMap::new();
//         d.add( "id", &self.id );
//         d.add( "time", &self.creation_time.sec );
//         d.add( "sender", &self.sender_name );
//         d.add( "subject", &self.subject );
//         d.add( "body", &self.body );
//         d.add( "readed", &self.readed );
//         Json::Object( d )
//     }
// }
