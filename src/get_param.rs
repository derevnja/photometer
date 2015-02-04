use params_body_parser::{ ParamsBody };
use std::str;
use super::err_msg;
use std::str::FromStr;
use types::{ CommonResult, Id };
use time;
use time::Timespec;
use iron::prelude::Request;

pub trait GetParamable {
    fn get_param( &self, prm: &str ) -> CommonResult<&str>;
    fn get_param_bin( &self, prm: &str ) -> CommonResult<&[u8]>;
    fn get_param_uint( &self, prm: &str ) -> CommonResult<usize>;
    fn get_param_id( &self, prm: &str ) -> CommonResult<Id>;
    fn get_param_time( &self, prm: &str ) -> CommonResult<Timespec>;
    fn get_params( &self, prm: &str ) -> CommonResult<&Vec<String>>;
}

//TODO: проверить на следующей версии раста, а пока ICE =(
/*pub trait GetManyParams {
    fn get_prm<'a, T: FromParams<'a>>( &'a self, prm: &str ) -> CommonResult<T>;
    fn get2params<'a, T1: FromParams<'a>, T2: FromParams<'a>>( &'a self, prm1: &str, prm2: &str ) -> CommonResult<(T1, T2)>;
    fn get3params<'a, T1: FromParams<'a>, T2: FromParams<'a>, T3: FromParams<'a>>( 
        &'a self, prm1: &str, prm2: &str, prm3: &str ) -> CommonResult<(T1, T2, T3)>;
}*/

pub trait FromParams<'a> {
    fn from_params( params: &'a GetParamable, prm: &str ) -> CommonResult<Self>;
}

impl<'a> GetParamable for Request<'a> {
    //инкапсулирует поиск параметра сначало в текстовом виде, потом в бинарном
    fn get_param( &self, prm: &str ) -> CommonResult<&str> {
        match self.parameter( prm ) {
            Some( s ) => Ok( s.as_slice() ),
            None => match self.bin_parameter( prm ) {
                Some( b ) => match str::from_utf8( b ) {
                    Ok( s ) => Ok( s ),
                    Err( _ ) => Err( err_msg::not_a_string_param( prm ) )
                },
                None => Err( err_msg::param_not_found( prm ) )
            }
        }
    }
    fn get_params( &self, prm: &str ) -> CommonResult<&Vec<String>> {
        self.parameters( prm )
            .ok_or( err_msg::param_not_found( prm ) )
    }
    fn get_param_bin( &self, prm: &str ) -> CommonResult<&[u8]> {
        self.bin_parameter( prm ).ok_or( err_msg::invalid_type_param( prm ) )
    }
    fn get_param_uint( &self, prm: &str ) -> CommonResult<usize> {
        self.get_param( prm )
            .and_then( |s| FromStr::from_str( s ).ok_or( err_msg::invalid_type_param( prm ) ) )
    }
    fn get_param_id( &self, prm: &str ) -> CommonResult<Id> {
        self.get_param( prm )
            .and_then( |s| FromStr::from_str( s ).ok_or( err_msg::invalid_type_param( prm ) ) ) 
    }

    fn get_param_time( &self, prm: &str ) -> CommonResult<Timespec> {
        self.get_param( prm )
            .and_then( |s| time::strptime( s, "%Y.%m.%d %k:%M:%S" )
                .map_err( |_| err_msg::parsing_error_param( prm ) )
                .map( |t| t.to_timespec() )
            )
    }
}

/*impl<'a, Params: GetParamable> GetManyParams for Params {
    fn get_prm<'x, T: FromParams<'x>>( &'x self, prm: &str ) -> CommonResult<T> {
        FromParams::from_params( self, prm )
    }
    fn get2params<'a, T1: FromParams<'a>, T2: FromParams<'a>>( &'a self, prm1: &str, prm2: &str ) -> CommonResult<(T1, T2)> {
        match FromParams::from_params( self, prm1 ) {
            Ok( p1 ) => match FromParams::from_params( self, prm2 ) {
                Ok( p2 ) => Ok( (p1, p2) ),
                Err( e ) => Err( e )
            },
            Err( e ) => Err( e )
        }
    }

    fn get3params<'a, T1: FromParams<'a>, T2: FromParams<'a>, T3: FromParams<'a>>( 
        &'a self, prm1: &str, prm2: &str, prm3: &str ) -> CommonResult<(T1, T2, T3)> {
        match FromParams::from_params( self, prm1 ) {
            Ok( p1 ) => match FromParams::from_params( self, prm2 ) {
                Ok( p2 ) => match FromParams::from_params( self, prm3 ) {
                    Ok( p3 ) => Ok( ( p1, p2, p3 ) ),
                    Err( e ) => Err( e )
                },
                Err( e ) => Err( e )
            },
            Err( e ) => Err( e )
        }   
    }
}*/

impl<'a> FromParams<'a> for &'a [u8] {
    fn from_params( params: &'a GetParamable, prm: &str ) -> CommonResult<&'a [u8]> {
        params.get_param_bin( prm )
    }   
}

impl<'a> FromParams<'a> for &'a str {
    fn from_params( params: &'a GetParamable, prm: &str ) -> CommonResult<&'a str> {
        params.get_param( prm )
    }   
}

impl<'a, T: FromStr> FromParams<'a> for T {
    fn from_params( params: &'a GetParamable, prm: &str ) -> CommonResult<T> {
        params.get_param( prm )
            .and_then( |s| FromStr::from_str( s ).ok_or( err_msg::invalid_type_param( prm ) ) )
    }
}
