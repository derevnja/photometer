use mysql::conn::pool::{ MyPooledConn };
use mysql::error::{ MyResult };
use mysql::value::{ from_value };
use types::{ Id, CommonResult };
use std::fmt::Show;

pub trait DbUsers {
    /// выбирает id пользователя по имени и паролю
    fn get_user( &mut self, name: &str, pass: &str ) -> CommonResult<Option<Id>>;
    /// добавляет нового пользователя в БД
    fn add_user( &mut self, name: &str, pass: &str ) -> CommonResult<()>;
    /// проверяет наличие имени в БД
    fn user_exists(&mut self, name: &str) -> CommonResult<bool>;
    fn user_id_exists(&mut self, id: Id ) -> CommonResult<bool>;
}

impl DbUsers for MyPooledConn {
    /// выбирает id пользователя по имени и паролю
    fn get_user( &mut self, name: &str, pass: &str ) -> CommonResult<Option<Id>> {
        get_user_impl( self, name, pass )
            .map_err( |e| fn_failed( "get_user", e ) )
    }
    
    /// добавляет нового пользователя в БД
    fn add_user( &mut self, name: &str, pass: &str ) -> CommonResult<()> {
        let name = name.to_string();
        let pass = pass.to_string();
        self.prepare( "INSERT INTO users (login, password) VALUES(?, ?);" )
            .and_then( |ref mut stmt| stmt.execute( &[ &name, &pass ] ).and( Ok( () ) ) )
            .map_err( |e| fn_failed( "add_user", e ) )
    }
    /// проверяет наличие имени в БД
    fn user_exists(&mut self, name: &str,) -> CommonResult<bool> {
        user_exists_impl( self, name )
            .map_err( |e| fn_failed( "user_exists", e ) )
    }
    /// проверяет наличие имени в БД
    fn user_id_exists(&mut self, id: Id ) -> CommonResult<bool> {
        user_id_exists_impl( self, id )
            .map_err( |e| fn_failed( "user_id_exists", e ) )  
    }
    
}

fn fn_failed<E: Show>( fn_name: &str, e: E ) -> String {
    format!( "DbUsers func '{}' failed: {}", fn_name, e )
}

fn get_user_impl( conn: &mut MyPooledConn, name: &str, pass: &str ) -> MyResult<Option<Id>> {
    let name = name.to_string(); // помогает убрать internal compiler error
    let pass = pass.to_string();
    let mut stmt = try!( conn.prepare( "select id from users where login=? and password=?" ) );
    let mut sql_result = try!( stmt.execute( &[ &name, &pass ] ) );
    sql_result.next().map_or( Ok( None ),
        |row| row.and_then( |r| Ok( Some( from_value::<Id>( &r[0] ) ) ) )
    )
}

fn user_exists_impl( conn: &mut MyPooledConn, name: &str  ) -> MyResult<bool> {
    let name = name.to_string();
    let mut stmt = try!( conn.prepare( "select id from users where login=?" ) );
    let sql_result = try!( stmt.execute( &[ &name ] ) );
    Ok( sql_result.count() == 1 )
}

fn user_id_exists_impl( conn: &mut MyPooledConn, id: Id  ) -> MyResult<bool> {
    let mut stmt = try!( conn.prepare( "select id from users where id=?" ) );
    let sql_result = try!( stmt.execute( &[ &id ] ) );
    Ok( sql_result.count() == 1 )
}