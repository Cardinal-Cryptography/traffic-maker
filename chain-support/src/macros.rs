/// `clone_args` expects non-empty sequence of arguments to be cloned.
/// There are 4 handled types of argument:
/// - *variable passed by value* (`$i:ident`): this one is simply cloned
///   to a new variable with the same name, so this snippet:
///   ```no_run
///     use chain_support::clone_args;   
///
///     let x = 5;
///     clone_args!(x);
///   ```
///   is equivalent to:
///   ```no_run
///     let x = 5;
///     let x = x.clone();
///   ```
///
/// - *variable passed by reference* (`& $i:ident`): this is handled exactly
///   the same as in the previous case, no difference;
///
/// - *field passed by value* (`$self:ident . $i:ident`): here we create a new
///   variable under the field name, so this snippet:
///   ```no_run
///     use chain_support::clone_args;    
///
///     struct S { f: usize }
///     let s = S { f: 5 };
///     clone_args!(s.f);
///   ```
///   is equivalent to:
///   ```no_run
///     struct S { f: usize }
///     let s = S { f: 5 };
///     let f = s.f.clone();
///   ```
///
/// - *field passed by reference* (`& $self:ident . $i:ident`): this is handled
///   exactly the same as in the previous case, no difference;
///
/// *Note:* Of course these rules are not perfectly safe, e.g. `ident` will
/// be matched against keyword, which will fail to compile. Please use it
/// with the simple rules mentioned above.
#[macro_export]
macro_rules! clone_args {
    // variable as a single argument
    ($(&)? $i:ident) => {
        let $i = $i.clone();
    };

    // field as a single argument
    ($(&)? $self:ident . $i:ident) => {
        let $i = $self.$i.clone();
    };

    // variable as the first argument
    ($(&)? $i:ident, $($rest:tt)*) => {
        let $i = $i.clone();
        $crate::clone_args!($($rest)*);
    };

    // field as the first argument
    ($(&)? $self:ident . $i:ident, $($rest:tt)*) => {
        let $i = $self.$i.clone();
        $crate::clone_args!($($rest)*);
    };
}

/// This macro is intended to be used after calling `clone_args` macro.
///
/// `exchange_args_with_cloned` expects (mainly) a sequence of arguments that should
/// be exchanged with their counterparts created by `clone_args`. To do so,
/// it has to perform 'output accumulation', i.e. as the first argument,
/// `exchange_args_with_cloned` takes accumulated result, which effectively is a tuple;
/// Hence, for the initial call you would probably pass `()` here.
/// If a single argument is passed, the result will be a singleton tuple.
///
/// The accumulator is separated with `;` from target args.
///
/// Similarly to `clone_args`, there are 4 categories of expected arguments:
/// - *variable passed by value* (`$i:ident`): this one will be translated to the same name
/// - *variable passed by reference* (`& $i:ident`): this one will be translated to the same
///   name, however, the counterpart will be taken by reference
/// - *field passed by value* (`$self:ident . $i:ident`): this one will be translated to the
///   field name
/// - *field passed by reference* (`& $self:ident . $i:ident`): this one will be translated
///   to the field name, however, the counterpart will be taken by reference
///
/// For example, this snippet:
/// ```no_run
///     use chain_support::{clone_args, exchange_args_with_cloned};    
///
///     struct S { f1: usize, f2: usize }
///     let s = S { f1: 1, f2: 2 };
///     let x = 3;
///     let y = 4;
///
///     clone_args!(x, &y, s.f1, &s.f2);
///     exchange_args_with_cloned!((); x, &y, s.f1, &s.f2);
/// ```
/// is equivalent to:
/// ```no_run
///     struct S { f1: usize, f2: usize }
///     let s = S { f1: 1, f2: 2 };
///     let x = 3;
///     let y = 4;
///
///     let x = x.clone();
///     let y = y.clone();
///     let f1 = s.f1.clone();
///     let f2 = s.f2.clone();
///     (x, &y, f1, &f2);
/// ```
#[macro_export]
macro_rules! exchange_args_with_cloned {
    // there is nothing more to do
    ($acc:expr;) => {
        $acc
    };

    // the last argument is a variable passed by value
    (($($acc:tt)*); $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i,);
        )
    };

    // the last argument is a variable passed by reference
    (($($acc:tt)*); & $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i,);
        )
    };

    // the last argument is a field passed by value
    (($($acc:tt)*); $self:ident . $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i);
        )
    };

    // the last argument is a field passed by reference
    (($($acc:tt)*); & $self:ident . $i:ident) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i);
        )
    };

    // the first argument is a variable passed by value
    (($($acc:tt)*); $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i,);
            $($rest)*
        )
    };

    // the first argument is a variable passed by reference
    (($($acc:tt)*); & $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i,);
            $($rest)*
        )
    };

    // the first argument is a field passed by value
    (($($acc:tt)*); $self:ident . $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* $i,);
            $($rest)*
        )
    };

    // the first argument is a field passed by reference
    (($($acc:tt)*); & $self:ident . $i:ident, $($rest:tt)*) => {
        $crate::exchange_args_with_cloned!(
            ($($acc)* &$i,);
            $($rest)*
        )
    };
}

/// `do_async` enables running blocking task within an asynchronous environment in a convenient way.
/// It takes the target function: `action` (unfortunately, it cannot be an associated method)
/// and its arguments. `action` is executed with `tokio::task::spawn_blocking`, so the
/// corresponding dependency has to be included in the caller's crate.
///
/// As delegating work to a new blocking thread has to capture arguments, that are expected
/// to have possibly longer lifetime that the passed ones, firstly, we have to clone them.
/// We do that with `clone_args` macro.
/// Then, we pass the cloned objects to the action. The problem is that we have to pass them
/// in a tuple (actually, there is no way of 'inlining' them into method invocation), so we have
/// to use unstable feature `fn_traits`. Hence, you have to include `#![feature(fn_traits)]`
/// rule in caller's crate root.
///
/// The permissible arguments are: variable, variable passed by reference, field and field
/// passed by reference.
///
/// For example, this snippet:
/// ```no_run
///     use chain_support::{do_async};    
///
///     fn fun(a: u8, b: usize, c: &usize) {}
///
///     struct S { f1: usize, f2: usize }
///     let s = S { f1: 1, f2: 2 };
///     let x = 3u8;
///     
///     do_async!(f, x, s.f1, &s.f2)?;
/// ```
/// is equivalent to:
/// ```no_run
///     fn fun(a: u8, b: usize, c: &usize) {}
///
///     struct S { f1: usize, f2: usize }
///     let s = S { f1: 1, f2: 2 };
///     let x = 3u8;
///
///     {
///          use tokio::task::spawn_blocking;
///
///          let x = x.clone();
///          let f1 = s.f1.clone();
///          let f2 = s.f2.clone();
///          spawn_blocking(move || f.call((x, f1, &f2))).await
///     }?
/// ```
///
/// Returns `tokio::runtime::task::Result<T>`, where `T` is the return type for `action`.
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
