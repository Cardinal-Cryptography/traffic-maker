#[macro_export]
macro_rules! clone_args {
    ($(&)? $i:ident) => {
        let $i = $i.clone();
    };

    ($(&)? $self:ident . $i:ident) => {
        let $i = $self.$i.clone();
    };

    ($e:expr) => {};

    ($(&)? $i:ident, $($rest:tt)*) => {
        let $i = $i.clone();
        $crate::clone_args!($($rest)*);
    };

    ($(&)? $self:ident . $i:ident, $($rest:tt)*) => {
        let $i = $self.$i.clone();
        $crate::clone_args!($($rest)*);
    };

    ($e:expr, $($rest:tt)*) => {
        $crate::clone_args!($($rest)*);
    };
}

#[macro_export]
macro_rules! exchange_args_with_cloned {
    ($acc:expr;) => {
        $acc
    };

    (($($acc:tt)*); $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i);
        )
    };

    (($($acc:tt)*); & $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i);
        )
    };

    (($($acc:tt)*); $self:ident . $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i);
        )
    };

    (($($acc:tt)*); & $self:ident . $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i);
        )
    };

    (($($acc:tt)*); $e:expr) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $e);
        )
    };

    (($($acc:tt)*); $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i,);
            $($rest)*
        )
    };

    (($($acc:tt)*); & $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i,);
            $($rest)*
        )
    };

    (($($acc:tt)*); $self:ident . $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i,);
            $($rest)*
        )
    };

    (($($acc:tt)*); & $self:ident . $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i,);
            $($rest)*
        )
    };

    (($($acc:tt)*); $e:expr, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $e,);
            $($rest)*
        )
    };
}

#[macro_export]
macro_rules! do_async {
    ($action:ident, $($arg:tt)*) => {
        {
            use tokio::task::spawn_blocking;

            $crate::clone_args!($($arg)*);
            spawn_blocking(move || $action.call($crate::exchange_args_with_cloned!((); $($arg)*))).await
        }
    };
}
