pub(crate) use const_format::str_replace;
pub(crate) macro str_replace_multiple {

    ( $original:expr $( , [ $(,)? ] )? $(,)? ) => { $original },

    ( $original:expr , [ ( $aa:expr , $ab:expr $(,)? ) $( , ( $ba:expr , $bb:expr $(,)? ) )* $(,)? ] $(,)? ) => {
        str_replace_multiple!( str_replace!( $original , $aa , $ab ) , [ $( ( $ba , $bb , ) , )* ] , )
    }

}
