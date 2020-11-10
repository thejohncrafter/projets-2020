
#[macro_export]
macro_rules! regexp {
    (_) => {$crate::lexer::Regexp::Character($crate::lexer::Character::Any)};
    (alpha) => {$crate::lexer::Regexp::Character($crate::lexer::Character::Alpha)};
    (num) => {$crate::lexer::Regexp::Character($crate::lexer::Character::Num)};
    ([$c:expr]) => {$crate::lexer::Regexp::Character($crate::lexer::Character::Char($c))};
    (epsilon) => {$crate::lexer::Regexp::Epsilon};
    
    (($l:tt | $r:tt)) => {
        $crate::lexer::Regexp::Union(Box::new(regexp!($l)), Box::new(regexp!($r)))
    };
    (($l:tt & $r:tt)) => {
        $crate::lexer::Regexp::Concat(Box::new(regexp!($l)), Box::new(regexp!($r)))
    };
    (($e:tt *)) => {
        $crate::lexer::Regexp::Star(Box::new(regexp!($e)))
    };
}

