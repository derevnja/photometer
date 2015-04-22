use mysql::conn::pool::{ MyPooledConn };
use mysql::error::{ MyResult };
use mysql::value::{ from_value, from_value_opt, ToValue, FromValue, Value };
use types::{ Id, PhotoInfo, ImageType, CommonResult, EmptyResult, CommonError };
use time::{ Timespec };
use database::Database;
use std::fmt::Display;

pub trait DbPhotos {
    /// добавление фотографии в галлерею пользователя
    fn add_photo( &mut self, user_id: Id, info: &PhotoInfo ) -> CommonResult<()>;
    /// получение информации о фото
    fn get_photo_info( &mut self, photo_id: Id ) -> CommonResult<Option<(String, PhotoInfo)>>;
    ///возвращает список описаний фоточек
    fn get_photo_infos( &mut self, owner_id: Id, start: Timespec, end: Timespec, offset: u32, count: u32 ) -> CommonResult<Vec<PhotoInfo>>;
    ///вычисляет кол-во фоток пользователя за опеределнный период
    fn get_photo_infos_count( &mut self, owner_id: Id, start: Timespec, end: Timespec ) -> CommonResult<u32>;
    ///переименование фотографии
    fn rename_photo( &mut self, photo_id: Id, newname: &str ) -> CommonResult<()>;
}

pub fn create_tables( db: &Database ) -> EmptyResult {
    db.execute(
        "CREATE TABLE IF NOT EXISTS `images` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `owner_id` int(4) unsigned DEFAULT '0',
            `upload_time` int(11) NOT NULL DEFAULT '0',
            `type` enum( 'jpg', 'png' ) NOT NULL DEFAULT 'jpg',
            `width` int(4) unsigned DEFAULT '0',
            `height` int(4) unsigned DEFAULT '0',
            `name` varchar(64) NOT NULL DEFAULT '',
            `iso` int(11) unsigned DEFAULT '0',
            `shutter_speed` int(11) DEFAULT '0',
            `aperture` decimal(8,4) NOT NULL DEFAULT '0',
            `focal_length` int(4) unsigned DEFAULT '0',
            `focal_length_35mm` int(4) unsigned DEFAULT '0',
            `camera_model` varchar(64) NOT NULL DEFAULT '',
            PRIMARY KEY ( `id` ),
            KEY `owner_image` ( `owner_id`, `upload_time` )
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
        ",
        "db::photos::create_tables"
    )
}

const ISO_DEFAULT: u32 = 0;
const SHUTTER_SPEED_DEFAULT: i32 = 0;
const APERTURE_DEFAULT: f32 = 0.;
const FOCAL_LENGTH_DEFAULT: u16 = 0;
const FOCAL_LENGTH_35MM_DEFAULT: u16 = 0;
const CAMERA_MODEL_DEFAULT: &'static str = "";


impl DbPhotos for MyPooledConn {
	/// добавление фотографии в галлерею пользователя
    fn add_photo( &mut self, user_id: Id, info: &PhotoInfo ) -> CommonResult<()> {
        add_photo_impl( self, user_id, info )
            .map_err( |e| fn_failed( "add_photo", e ) )
    }

    /// получение информации о фото
    fn get_photo_info( &mut self, photo_id: Id ) -> CommonResult<Option<(String, PhotoInfo)>> {
        get_photo_info_impl( self, photo_id )
            .map_err( |e| fn_failed( "get_photo_info", e ) )
    }

    ///возвращает список описаний фоточек
    fn get_photo_infos( &mut self, owner_id: Id, start: Timespec, end: Timespec, offset: u32, count: u32 ) -> CommonResult<Vec<PhotoInfo>> {
        get_photo_infos_impl( self, owner_id, start, end, offset, count )
            .map_err( |e| fn_failed( "get_photo_infos", e ) )
    }

    ///вычисляет кол-во фоток пользователя за опеределнный период
    fn get_photo_infos_count( &mut self, owner_id: Id, start: Timespec, end: Timespec ) -> CommonResult<u32> {
        get_photo_infos_count_impl( self, owner_id, start, end )
            .map_err( |e| fn_failed( "get_photo_infos_count", e ) )
    }

    ///переименование фотографии
    fn rename_photo( &mut self, photo_id: Id, newname: &str ) -> CommonResult<()> {
        rename_photo_impl( self, photo_id, newname )
            .map_err( |e| fn_failed( "rename_photo", e ) )
    }

}

fn fn_failed<E: Display>( fn_name: &str, e: E ) -> CommonError {
    CommonError( format!( "DbPhotos func '{}' failed: {}", fn_name, e ) )
}

fn add_photo_impl( conn: &mut MyPooledConn, user_id: Id, info: &PhotoInfo ) -> MyResult<()> {
    let mut stmt = try!( conn.prepare(
        "insert into images (
        owner_id,
        upload_time,
        type,
        width,
        height,
        name,
        iso,
        shutter_speed,
        aperture,
        focal_length,
        focal_length_35mm,
        camera_model )
        values( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )" )
    );
    try!( stmt.execute( &[
        &user_id,
        &info.upload_time,
        &info.image_type,
        &info.width,
        &info.height,
        &info.name,
        &info.iso.unwrap_or( ISO_DEFAULT ),
        &info.shutter_speed.unwrap_or( SHUTTER_SPEED_DEFAULT ),
        &info.aperture.unwrap_or( APERTURE_DEFAULT ),
        &info.focal_length.unwrap_or( FOCAL_LENGTH_DEFAULT ),
        &info.focal_length_35mm.unwrap_or( FOCAL_LENGTH_35MM_DEFAULT ),
        info.camera_model.as_ref().unwrap_or( &CAMERA_MODEL_DEFAULT.to_string() )
    ]));
    Ok( () )
}

fn get_photo_info_impl( conn: &mut MyPooledConn, photo_id: Id ) -> MyResult<Option<(String, PhotoInfo)>> {
    let mut stmt = try!( conn.prepare( "SELECT
        u.login,
        i.id,
        i.upload_time,
        i.type,
        i.width,
        i.height,
        i.name,
        i.iso,
        i.shutter_speed,
        i.aperture,
        i.focal_length,
        i.focal_length_35mm,
        i.camera_model
        FROM images AS i LEFT JOIN users AS u ON ( u.id = i.owner_id )
        WHERE u.id IS NOT NULL AND i.id = ?" )
    );
    let mut sql_result = try!( stmt.execute( &[ &photo_id ] ) );
    match sql_result.next() {
        None => Ok( None ),
        Some( sql_row ) => {
            let row_data = try!( sql_row );
            let mut values = row_data.iter();
            Ok ( Some ( (
                from_value( values.next().unwrap() ),
                read_photo_info( &mut values )
            ) ) )
        }
    }
}

fn get_photo_infos_impl(
    conn: &mut MyPooledConn,
    owner_id: Id,
    start: Timespec,
    end: Timespec,
    offset: u32,
    count: u32
) -> MyResult<Vec<PhotoInfo>>
{
    let mut stmt = try!( conn.prepare( "SELECT
       id,
       upload_time,
       type,
       width,
       height,
       name,
       iso,
       shutter_speed,
       aperture,
       focal_length,
       focal_length_35mm,
       camera_model
       FROM images
       WHERE owner_id = ? AND upload_time BETWEEN ? AND ?
       ORDER BY upload_time ASC
       LIMIT ? OFFSET ?;
    " ) );
    let result = try!( stmt.execute( &[ &owner_id, &start.sec, &end.sec, &count, &offset ] ) );
    //что-то с преобразованием на лету через собственный итертор я подупрел =(, пришлось тупо собирать в новый массив
    let photos : Vec<_> = result.filter_map( |sql_row|
        sql_row.ok().map( |sql_values| {
            let mut values = sql_values.iter();
            read_photo_info( &mut values )
        })
    ).collect();
    Ok( photos )
}

fn get_photo_infos_count_impl( conn: &mut MyPooledConn, owner_id: Id, start: Timespec, end: Timespec ) -> MyResult<u32> {
    let mut stmt = try!( conn.prepare( "SELECT COUNT(id) FROM images WHERE owner_id = ? AND upload_time BETWEEN ? AND ?" ) );
    let mut result = try!( stmt.execute( &[ &owner_id, &start.sec, &end.sec ] ) );
    let sql_row = try!( result.next().unwrap() );
    Ok( from_value( &sql_row[ 0 ] ) )
}

fn rename_photo_impl( conn: &mut MyPooledConn, photo_id: Id, newname: &str ) -> MyResult<()> {
    let newname = newname.to_string();
    let mut stmt = try!( conn.prepare( "UPDATE images SET name=? WHERE id=?" ) );
    let _ = try!( stmt.execute( &[ &newname, &photo_id ] ) );
    Ok( () )
}

fn read_photo_info<'a, I: Iterator<Item = &'a Value>>( values: &mut I ) -> PhotoInfo
{
    PhotoInfo {
        id: from_value( values.next().unwrap() ),
        upload_time: from_value( values.next().unwrap() ),
        image_type: from_value( values.next().unwrap() ),
        width: from_value( values.next().unwrap() ),
        height: from_value( values.next().unwrap() ),
        name: from_value( values.next().unwrap() ),
        iso: if_not( from_value( values.next().unwrap() ), ISO_DEFAULT ),
        shutter_speed: if_not( from_value( values.next().unwrap() ), SHUTTER_SPEED_DEFAULT ),
        aperture: if_not( from_value( values.next().unwrap() ), APERTURE_DEFAULT ),
        focal_length: if_not( from_value( values.next().unwrap() ), FOCAL_LENGTH_DEFAULT ),
        focal_length_35mm: if_not( from_value( values.next().unwrap() ), FOCAL_LENGTH_35MM_DEFAULT ),
        camera_model: if_not( from_value( values.next().unwrap() ), CAMERA_MODEL_DEFAULT.to_string() )
    }
}

fn if_not<T: PartialEq>( val: T, bad_value: T ) -> Option<T> {
    if val != bad_value {
        Some( val )
    }
    else {
        None
    }
}

const JPEG_STR : &'static str = "jpg";
const PNG_STR : &'static str = "png";

impl ToValue for ImageType {
    fn to_value(&self) -> Value {
        match self {
            &ImageType::Jpeg => JPEG_STR.to_value(),
            &ImageType::Png => PNG_STR.to_value()
        }
    }
}

impl FromValue for ImageType {
    fn from_value(v: &Value) -> ImageType {
        from_value_opt::<ImageType>( v ).expect( "fail converting ImageType from db value!" )
    }
    fn from_value_opt(v: &Value) -> Option<ImageType> {
        from_value_opt::<String>( v )
            .and_then( |string| {
                let s: &str = &string;
                match s {
                    JPEG_STR => Some( ImageType::Jpeg ),
                    PNG_STR => Some( ImageType::Png ),
                    _ => None
                }
            })
    }
}
