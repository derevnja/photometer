use mysql::conn::pool::{ MyPooledConn };
use mysql::error::{ MyResult };
use mysql::value::{ from_value };
use types::{ Id, EmptyResult, CommonResult };
use std::fmt::Display;
use database::Database;

pub trait DbPublication {
    /// публикует фото
    fn public_photo( &mut self, scheduled: Id, group: Id, user: Id, photo: Id, visible: bool ) -> EmptyResult;
    /// открывает на просмотр определнную группу фото
    fn make_publication_visible( &mut self, scheduled: Id, group: Id ) -> EmptyResult;
    /// кол-во уже опубликованных фото
    fn get_published_photo_count( &mut self, scheduled: Id, group: Id ) -> CommonResult<u32>;
    /// возвращает идентификаторы пользователей которые не проголосовали
    fn get_unpublished_users( &mut self, scheduled: Id, group: Id ) -> CommonResult<Vec<(Id, String)>>;
}

pub fn create_tables( db: &Database ) -> EmptyResult {
    db.execute(
        "CREATE TABLE IF NOT EXISTS `publication` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `scheduled_id` bigint(20) NOT NULL DEFAULT '0',
            `group_id` bigint(20) NOT NULL DEFAULT '0',
            `user_id` bigint(20) NOT NULL DEFAULT '0',
            `photo_id` bigint(20) NOT NULL DEFAULT '0',
            `visible` BOOL NOT NULL DEFAULT false,
            PRIMARY KEY ( `id` ),
            KEY `group_publication_idx` ( `group_id`, `scheduled_id`, `visible` ) USING BTREE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
        ",
        "db::publications::create_tables"
    )
}

impl DbPublication for MyPooledConn {
    /// публикует фото
    fn public_photo( &mut self, scheduled: Id, group: Id, user: Id, photo: Id, visible: bool ) -> EmptyResult {
        public_photo_impl( self, scheduled, group, user, photo, visible )
            .map_err( |e| fn_failed( "public_photo", e ) )
        
    }
    /// открывает на просмотр определнную группу фото
    fn make_publication_visible( &mut self, scheduled: Id, group: Id ) -> EmptyResult {
        make_publication_visible_impl( self, scheduled, group )
            .map_err( |e| fn_failed( "make_publication_visible", e ) )
    }

    /// кол-во уже опубликованных фото
    fn get_published_photo_count( &mut self, scheduled: Id, group: Id ) -> CommonResult<u32> {
        get_published_photo_count_impl( self, scheduled, group )
            .map_err( |e| fn_failed( "get_publicated_photo_count", e ) )
    }

    /// возвращает идентификаторы пользователей которые не проголосовали
    fn get_unpublished_users( &mut self, scheduled: Id, group: Id ) -> CommonResult<Vec<(Id, String)>> {
        get_unpublished_users_impl( self, scheduled, group )
            .map_err( |e| fn_failed( "get_unpublished_users", e ) )
    }
}

fn fn_failed<E: Display>( fn_name: &str, e: E ) -> String {
    format!( "DbPublication {} failed: {}", fn_name, e )
}

fn public_photo_impl( conn: &mut MyPooledConn, scheduled: Id, group: Id, user: Id, photo: Id, visible: bool ) -> MyResult<()> {
    let mut stmt = try!( conn.prepare("
        INSERT INTO publication (
            scheduled_id,
            group_id,
            user_id,
            photo_id,
            visible
        )
        VALUES( ?, ?, ?, ?, ? )
        ON DUPLICATE KEY UPDATE photo_id=?
    "));  
    try!( stmt.execute( &[
        &scheduled,
        &group,
        &user,
        &photo,
        &visible,
        &photo
    ]));
    Ok( () )
}

fn make_publication_visible_impl( conn: &mut MyPooledConn, scheduled: Id, group: Id ) -> MyResult<()> {
    let mut stmt = try!( conn.prepare( "
        UPDATE publication 
        SET visible=true
        WHERE scheduled_id = ? AND group_id = ?
    "));

    try!( stmt.execute( &[ &scheduled, &group ] ) );
    Ok( () )
}

fn get_published_photo_count_impl( conn: &mut MyPooledConn, scheduled: Id, group: Id ) -> MyResult<u32> {
    let mut stmt = try!( conn.prepare( "SELECT COUNT(id) FROM publication WHERE scheduled_id=? AND group_id=?" ) );
    let mut result = try!( stmt.execute( &[ &scheduled, &group ] ) );
    let row = try!( result.next().unwrap() );
    Ok( from_value( &row[ 0 ] ) )
}

fn get_unpublished_users_impl( conn: &mut MyPooledConn, scheduled: Id, group: Id ) -> MyResult<Vec<(Id, String)>> {
    let mut stmt = try!( conn.prepare( 
        "SELECT
            `g`.`user_id`, `u`.`login`
        FROM
            `group_members` AS `g`
        LEFT JOIN
            `users` AS `u` ON ( `u`.`id` = `g`.`user_id` )
        LEFT JOIN
            `publication` AS `p` ON ( `p`.`user_id` = `u`.`id` AND `p`.`scheduled_id` = ? )
        WHERE
            `g`.group_id = ?
            AND `p`.`id` IS NULL
    "));
    let mut result = try!( stmt.execute( &[ &scheduled, &group ] ) );
    let mut users = Vec::new();
    for row in result {
        let row = try!( row );
        users.push( ( from_value( &row[ 0 ] ), from_value( &row[ 1 ] ) ) );
    }
    Ok( users )
}