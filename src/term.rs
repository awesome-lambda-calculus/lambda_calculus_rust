//! [Lambda terms](https://en.wikipedia.org/wiki/Lambda_calculus#Lambda_terms)

pub use self::Notation::*;
pub use self::Term::*;
use self::TermError::*;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;

/// The character used to display lambda abstractions (a backslash).
#[cfg(feature = "backslash_lambda")]
pub const LAMBDA: char = '\\';

/// The character used to display lambda abstractions. The default is the Greek letter 'λ', but it
/// can also be set to a '\' (backslash) using `features = ["backslash_lambda"]`.
#[cfg(not(feature = "backslash_lambda"))]
pub const LAMBDA: char = 'λ';

/// An undefined term that can be used as a value returned by invalid/inapplicable operations, e.g.
/// obtaining an element of an empty list. Since this implementation uses De Bruijn indices greater
/// than zero, `Var(0)` will not occur naturally. It is displayed as `undefined`.
pub const UD: Term = Var(0);

/// The notation used for parsing and displaying purposes.
///
/// # Examples
/// ```
/// use lambda_calculus::combinators::S;
///
/// assert_eq!(&format!(  "{}", S()), "λa.λb.λc.a c (b c)"); // Classic notation
/// assert_eq!(&format!("{:?}", S()), "λλλ31(21)");          // DeBruijn index notation
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Notation {
    /// classic lambda calculus notation; used by `fmt::Display`
    Classic,
    /// De Bruijn indices; used by `fmt::Debug`
    DeBruijn,
}

/// A context holding a list of names for classic notation printing.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Context(Vec<String>);

impl Context {
    /// Creates a new `Context` from a slice of string-like items.
    ///
    /// This is the primary, most flexible constructor. It accepts anything
    /// that can be borrowed as a string slice, like `&[&str]` or `&[String]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lambda_calculus::term::Context;
    ///
    /// // Create from an array of &str
    /// let context1 = Context::new(&["a", "b", "c"]);
    ///
    /// // Create from a Vec<String>
    /// let names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    /// let context2 = Context::new(&names);
    ///
    /// assert_eq!(context1, context2);
    /// ```
    pub fn new<S: AsRef<str>>(namings: &[S]) -> Self {
        let owned = namings.iter().map(|s| s.as_ref().to_string()).collect();
        Context(owned)
    }

    /// Creates an empty context.
    pub fn empty() -> Self {
        vec![].into()
    }

    /// Returns an iterator over the names in the context, yielding `&str`.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.iter().map(|s| s.as_str())
    }

    /// Returns the number of names in the context.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the context contains no names.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if the context contains a name equivalent to the given value.
    ///
    /// This method is generic over `AsRef<str>`, so it can be called with
    /// a string slice (`&str`), a `String`, or other string-like types.
    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.iter().any(|item| item == name.as_ref())
    }

    /// Resolves a 1-based index to a free variable name from the context.
    ///
    /// The index is 1-based, where `1` refers to the first name defined in the context.
    /// Returns `None` if the index is 0 or out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lambda_calculus::term::Context;
    /// let ctx = Context::new(&["a", "b", "c"]);
    ///
    /// assert_eq!(ctx.resolve_free_var(1), Some("a"));
    /// assert_eq!(ctx.resolve_free_var(3), Some("c"));
    /// assert_eq!(ctx.resolve_free_var(0), None);
    /// assert_eq!(ctx.resolve_free_var(4), None);
    /// ```
    pub fn resolve_free_var(&self, idx: usize) -> Option<&str> {
        if idx == 0 {
            None
        } else {
            self.0.get(idx - 1).map(|s| s.as_str())
        }
    }
}

impl<S: AsRef<str>> From<&[S]> for Context {
    fn from(namings: &[S]) -> Self {
        Self::new(namings)
    }
}

impl From<Vec<String>> for Context {
    fn from(namings: Vec<String>) -> Self {
        Context(namings)
    }
}

/// A lambda term that is either a variable with a De Bruijn index, an abstraction over a term or
/// an applicaction of one term to another.
#[derive(PartialEq, Clone, Hash, Eq)]
pub enum Term {
    /// a variable
    Var(usize),
    /// an abstraction
    Abs(Box<Term>),
    /// an application
    App(Box<(Term, Term)>),
}

/// An error that can be returned when an inapplicable function is applied to a `Term`.
#[derive(Debug, PartialEq, Eq)]
pub enum TermError {
    /// the term is not a variable
    NotVar,
    /// the term is not an abstraction
    NotAbs,
    /// the term is not an application
    NotApp,
}

impl fmt::Display for TermError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TermError::NotVar => write!(f, "the term is not a variable",),
            TermError::NotAbs => write!(f, "the term is not an abstraction"),
            TermError::NotApp => write!(f, "the term is not an application"),
        }
    }
}

impl Error for TermError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Term {
    /// Returns a variable's De Bruijn index, consuming it in the process.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(Var(1).unvar(), Ok(1));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not a `Var`iable.
    pub fn unvar(self) -> Result<usize, TermError> {
        if let Var(n) = self {
            Ok(n)
        } else {
            Err(NotVar)
        }
    }

    /// Returns a reference to a variable's De Bruijn index.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(Var(1).unvar_ref(), Ok(&1));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not a `Var`iable.
    pub fn unvar_ref(&self) -> Result<&usize, TermError> {
        if let Var(ref n) = *self {
            Ok(n)
        } else {
            Err(NotVar)
        }
    }

    /// Returns a mutable reference to a variable's De Bruijn index.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(Var(1).unvar_mut(), Ok(&mut 1));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not a `Var`iable.
    pub fn unvar_mut(&mut self) -> Result<&mut usize, TermError> {
        if let Var(ref mut n) = *self {
            Ok(n)
        } else {
            Err(NotVar)
        }
    }

    /// Returns an abstraction's underlying term, consuming it in the process.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(abs(Var(1)).unabs(), Ok(Var(1)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `Abs`traction.
    pub fn unabs(self) -> Result<Term, TermError> {
        if let Abs(x) = self {
            Ok(*x)
        } else {
            Err(NotAbs)
        }
    }

    /// Returns a reference to an abstraction's underlying term.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(abs(Var(1)).unabs_ref(), Ok(&Var(1)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `Abs`traction.
    pub fn unabs_ref(&self) -> Result<&Term, TermError> {
        if let Abs(ref x) = *self {
            Ok(x)
        } else {
            Err(NotAbs)
        }
    }

    /// Returns a mutable reference to an abstraction's underlying term.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(abs(Var(1)).unabs_mut(), Ok(&mut Var(1)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `Abs`traction.
    pub fn unabs_mut(&mut self) -> Result<&mut Term, TermError> {
        if let Abs(ref mut x) = *self {
            Ok(x)
        } else {
            Err(NotAbs)
        }
    }

    /// Returns a pair containing an application's underlying terms, consuming it in the process.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).unapp(), Ok((Var(1), Var(2))));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn unapp(self) -> Result<(Term, Term), TermError> {
        if let App(boxed) = self {
            let (lhs, rhs) = *boxed;
            Ok((lhs, rhs))
        } else {
            Err(NotApp)
        }
    }

    /// Returns a pair containing references to an application's underlying terms.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).unapp_ref(), Ok((&Var(1), &Var(2))));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn unapp_ref(&self) -> Result<(&Term, &Term), TermError> {
        if let App(boxed) = self {
            let (ref lhs, ref rhs) = **boxed;
            Ok((lhs, rhs))
        } else {
            Err(NotApp)
        }
    }

    /// Returns a pair containing mutable references to an application's underlying terms.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).unapp_mut(), Ok((&mut Var(1), &mut Var(2))));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn unapp_mut(&mut self) -> Result<(&mut Term, &mut Term), TermError> {
        if let App(boxed) = self {
            let (ref mut lhs, ref mut rhs) = **boxed;
            Ok((lhs, rhs))
        } else {
            Err(NotApp)
        }
    }

    /// Returns the left-hand side term of an application. Consumes `self`.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).lhs(), Ok(Var(1)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn lhs(self) -> Result<Term, TermError> {
        if let Ok((lhs, _)) = self.unapp() {
            Ok(lhs)
        } else {
            Err(NotApp)
        }
    }

    /// Returns a reference to the left-hand side term of an application.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).lhs_ref(), Ok(&Var(1)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn lhs_ref(&self) -> Result<&Term, TermError> {
        if let Ok((lhs, _)) = self.unapp_ref() {
            Ok(lhs)
        } else {
            Err(NotApp)
        }
    }

    /// Returns a mutable reference to the left-hand side term of an application.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).lhs_mut(), Ok(&mut Var(1)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn lhs_mut(&mut self) -> Result<&mut Term, TermError> {
        if let Ok((lhs, _)) = self.unapp_mut() {
            Ok(lhs)
        } else {
            Err(NotApp)
        }
    }

    /// Returns the right-hand side term of an application. Consumes `self`.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).rhs(), Ok(Var(2)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn rhs(self) -> Result<Term, TermError> {
        if let Ok((_, rhs)) = self.unapp() {
            Ok(rhs)
        } else {
            Err(NotApp)
        }
    }

    /// Returns a reference to the right-hand side term of an application.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).rhs_ref(), Ok(&Var(2)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn rhs_ref(&self) -> Result<&Term, TermError> {
        if let Ok((_, rhs)) = self.unapp_ref() {
            Ok(rhs)
        } else {
            Err(NotApp)
        }
    }

    /// Returns a mutable reference to the right-hand side term of an application.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(app(Var(1), Var(2)).rhs_mut(), Ok(&mut Var(2)));
    /// ```
    /// # Errors
    ///
    /// Returns a `TermError` if `self` is not an `App`lication.
    pub fn rhs_mut(&mut self) -> Result<&mut Term, TermError> {
        if let Ok((_, rhs)) = self.unapp_mut() {
            Ok(rhs)
        } else {
            Err(NotApp)
        }
    }

    /// Returns `true` if `self` is a
    /// [supercombinator](https://en.wikipedia.org/wiki/Supercombinator).
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// let term1 = abs(app(Var(1), abs(Var(1)))); // λ 1 (λ 1)
    /// let term2 = app(abs(Var(2)), abs(Var(1))); // (λ 2) (λ 1)
    ///
    /// assert_eq!(term1.is_supercombinator(), true);
    /// assert_eq!(term2.is_supercombinator(), false);
    /// ```
    pub fn is_supercombinator(&self) -> bool {
        let mut stack = vec![(0usize, self)];

        while let Some((depth, term)) = stack.pop() {
            match term {
                Var(i) => {
                    if *i > depth || *i == 0 {
                        return false;
                    }
                }
                Abs(t) => stack.push((depth + 1, t)),
                App(boxed) => {
                    let (ref f, ref a) = **boxed;
                    stack.push((depth, f));
                    stack.push((depth, a))
                }
            }
        }
        true
    }

    /// Returns the maximum depth of lambda abstractions
    /// in the given `Term`.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// assert_eq!(abs(Var(1)).max_depth(), 1);
    /// ```
    pub fn max_depth(&self) -> u32 {
        match self {
            Var(_) => 0,
            Abs(t) => t.max_depth() + 1,
            App(boxed) => {
                let d0 = boxed.0.max_depth();
                let d1 = boxed.1.max_depth();
                d0.max(d1)
            }
        }
    }

    /// Returns `true` if `self` is structurally isomorphic to `other`.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// let term1 = abs(Var(1)); // λ 1
    /// let term2 = abs(Var(2)); // λ 2
    /// let term3 = abs(Var(1)); // λ 1
    ///
    /// assert_eq!(term1.is_isomorphic_to(&term2), false);
    /// assert_eq!(term1.is_isomorphic_to(&term3), true);
    ///
    /// ```
    pub fn is_isomorphic_to(&self, other: &Term) -> bool {
        match (self, other) {
            (Var(x), Var(y)) => x == y,
            (Abs(p), Abs(q)) => p.is_isomorphic_to(q),
            (App(p), App(q)) => p.0.is_isomorphic_to(&q.0) && p.1.is_isomorphic_to(&q.1),
            _ => false,
        }
    }

    /// Returns `true` if `self` has any free vairables.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::*;
    ///
    /// let with_freevar = abs(Var(2));     // λ 2
    /// let without_freevar = abs(Var(1));  // λ 1
    ///
    /// assert!(with_freevar.has_free_variables());
    /// assert!(!without_freevar.has_free_variables());
    pub fn has_free_variables(&self) -> bool {
        self.has_free_variables_helper(0)
    }

    fn has_free_variables_helper(&self, depth: usize) -> bool {
        match self {
            Var(x) => *x > depth || *x == 0,
            Abs(p) => p.has_free_variables_helper(depth + 1),
            App(p) => p.0.has_free_variables_helper(depth) || p.1.has_free_variables_helper(depth),
        }
    }

    /// Calculates the maximum index of any free variable in the term.
    ///
    /// The result corresponds to the number of names `Context` must supply to bind them all.
    pub fn max_free_index(&self) -> usize {
        self.max_free_index_helper(0)
    }

    fn max_free_index_helper(&self, depth: usize) -> usize {
        match self {
            Var(x) => x.saturating_sub(depth),
            Abs(p) => p.max_free_index_helper(depth + 1),
            App(p) => {
                p.0.max_free_index_helper(depth)
                    .max(p.1.max_free_index_helper(depth))
            }
        }
    }

    /// Returns a helper struct that allows displaying the term with a given context.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::{*, term::Context};
    ///
    /// let term = abs(Var(2)); // λa.b
    /// let ctx = Context::new(&["x"]); // Predefine "x" as a free variable
    ///
    /// // The context defines `Var(2)` as "x" instead of the default "b"
    /// assert_eq!(term.with_context(&ctx).to_string(), "λa.x");
    /// ```
    pub fn with_context<'a>(&'a self, ctx: &'a Context) -> impl fmt::Display + 'a {
        DisplayWithContext { term: self, ctx }
    }

    /// Return if has beta redex
    pub fn has_beta_redex(&self) -> bool {
        match self {
            Var(_) => false,
            Abs(term) => term.has_beta_redex(),
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                match t1 {
                    Abs(_) => return true,
                    _ => {}
                }
                t1.has_beta_redex() || t2.has_beta_redex()
            }
        }
    }

    /// Returns `true` if all variables in the term are bound by an abstraction at a depth less than `k`.
    pub fn lcat(&self, k: usize) -> bool {
        match self {
            Var(i) => *i < k,              // Check if the variable index is less than k.
            Abs(term) => term.lcat(k + 1), // Recursively check the body of the abstraction.
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.lcat(k) && t2.lcat(k) // Both the function and argument must satisfy the condition.
            }
        }
    }

    /// Returns `true` if all variables in the term are bound by an abstraction at a depth less than `0`
    pub fn lc(&self) -> bool {
        self.lcat(0)
    }

    /// Returns the number of abstractions and applications in the term
    pub fn fokker_size(&self) -> usize {
        match self {
            Var(_) => 0,
            Abs(term) => 1 + term.fokker_size(),
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                1 + t1.fokker_size() + t2.fokker_size()
            }
        }
    }

    fn count_bvar(&self, goal: usize) -> usize {
        match self {
            Var(i) => {
                if *i == goal {
                    1
                } else {
                    0
                }
            }
            Abs(term) => term.count_bvar(goal + 1),
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.count_bvar(goal) + t2.count_bvar(goal)
            }
        }
    }

    fn count_bvar_all(&self) -> usize {
        match self {
            Var(_) => 1,
            Abs(term) => term.count_bvar_all(),
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.count_bvar_all() + t2.count_bvar_all()
            }
        }
    }

    /// This is a decider
    pub fn only_one_var_used(&self) -> bool {
        match self {
            Var(_) => true,
            Abs(term) => {
                (term.count_bvar(0) == term.count_bvar_all())
                    || (term.count_bvar(0) == 0 && term.only_one_var_used())
            }
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.only_one_var_used() && t2.only_one_var_used()
            }
        }
    }

    /// Return if has eta redex
    pub fn has_eta_redex(&self) -> bool {
        match self {
            Var(_) => false,
            Abs(term) => match term.as_ref() {
                App(boxed) => {
                    let (ref t1, ref t2) = **boxed;
                    if *t2 == Var(0) && t1.count_bvar(0) == 0 {
                        true
                    } else {
                        term.has_eta_redex()
                    }
                }
                _ => term.has_eta_redex(),
            },
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.has_eta_redex() || t2.has_eta_redex()
            }
        }
    }

    /// Shifts free variables down by a given amount when a binder is removed.
    fn shift(&mut self, amount: isize, depth: usize) {
        match self {
            Term::Var(v) => {
                if *v >= depth {
                    *v = (*v as isize + amount) as usize;
                }
            }
            Term::Abs(inner) => inner.shift(amount, depth + 1),
            Term::App(inner) => {
                inner.0.shift(amount, depth);
                inner.1.shift(amount, depth);
            }
        }
    }

    /// Reduces the term using eta reduction.
    pub fn eta_reduce(self) -> Self {
        match self {
            Term::Var(_) => self,

            Term::App(inner) => {
                let (t1, t2) = *inner;
                Term::App(Box::new((t1.eta_reduce(), t2.eta_reduce())))
            }

            Term::Abs(inner) => {
                let reduced_inner = inner.eta_reduce();

                if let Term::App(app_inner) = reduced_inner {
                    let (mut m, arg) = *app_inner;

                    // Match the pattern: Abs(App(M, Var(0)))
                    if let Term::Var(0) = arg {
                        // If the variable '0' appears 0 times in M, it is not free in M.
                        if m.count_bvar(0) == 0 {
                            // Strip the Abs binder and shift outer free variables down
                            m.shift(-1, 0);
                            return m.eta_reduce();
                        }
                    }
                    // Reconstruct if reduction criteria aren't met
                    Term::Abs(Box::new(Term::App(Box::new((m, arg)))))
                } else {
                    Term::Abs(Box::new(reduced_inner))
                }
            }
        }
    }

    /// Returns a string representation of the term in Lean syntax.
    pub fn print_lean(&self) -> String {
        match self {
            Var(i) => format!(".bvar {i}"),
            Abs(term) => format!(".abs ({})", term.print_lean()),
            App(term) => {
                let (a, b) = term.as_ref();
                format!(".app ({}) ({})", a.print_lean(), b.print_lean())
            }
        }
    }

    /// Returns a string representation of the term in a custom encoding format
    pub fn encode(&self) -> String {
        match self {
            Var(i) => i.to_string(),
            Abs(term) => format!("L{}", term.encode()),
            App(ab) => {
                let (a, b) = ab.as_ref();
                format!("A{}{}", a.encode(), b.encode())
            }
        }
    }

    /// Decodes a list of characters into a Term.
    pub fn decode_fuel(chars: &[char]) -> Option<(Self, &[char])> {
        // Attempt to split the head and tail of the character slice.
        // If empty, returns None.
        let (head, bs) = chars.split_first()?;

        match head {
            'L' => {
                // The `?` operator acts exactly like the Lean `match ... with | some => ... | none => none`
                let (t, rest) = Self::decode_fuel(bs)?;
                Some((Term::Abs(Box::new(t)), rest))
            }
            'A' => {
                let (t1, rest1) = Self::decode_fuel(bs)?;
                let (t2, rest2) = Self::decode_fuel(rest1)?;
                Some((Term::App(Box::new((t1, t2))), rest2))
            }
            c @ '0'..='9' => {
                // Match bounds inclusively, then convert to a numeric value
                let val = c.to_digit(10).unwrap();
                Some((Term::Var(val.try_into().unwrap()), bs))
            }
            _ => None,
        }
    }

    /// Returns `true` if all variables in the term are `Var(0)`, i.e. the term is structurally isomorphic to `UD`.
    pub fn all0(&self) -> bool {
        match self {
            Var(0) => true,
            Var(_) => false,
            Abs(term) => term.all0(),
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.all0() && t2.all0()
            }
        }
    }

    /// Returns `true` if no variable index occurs more than once in the term.
    pub fn no_duplicate(&self) -> bool {
        match self {
            Var(_) => true,
            Abs(term) => term.count_bvar(0) < 2 && term.no_duplicate(),
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.no_duplicate() && t2.no_duplicate()
            }
        }
    }

    /// Returns `true` if every bound variable in the term is used.
    pub fn every_bvar_used(&self) -> bool {
        match self {
            Var(_) => true,
            Abs(term) => self.count_bvar(0) > 0 && term.every_bvar_used(),
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.every_bvar_used() && t2.every_bvar_used()
            }
        }
    }

    /// Returns `true` if the term can be represented using only two distinct variable indices, i.e. `Var(1)` and `Var(2)`.
    pub fn two_vars_are_enough(&self) -> bool {
        match self {
            Var(_) => true,
            Abs(term) => match term.as_ref() {
                Abs(term) => term.two_vars_are_enough(),
                _ => false,
            },
            App(boxed) => {
                let (ref t1, ref t2) = **boxed;
                t1.two_vars_are_enough() && t2.two_vars_are_enough()
            }
        }
    }
}

/// Wraps a `Term` in an `Abs`traction. Consumes its argument.
///
/// # Example
/// ```
/// use lambda_calculus::*;
///
/// assert_eq!(abs(Var(1)), Abs(Box::new(Var(1))));
/// ```
pub fn abs(term: Term) -> Term {
    Abs(Box::new(term))
}

/// Produces an `App`lication of two given `Term`s without any reduction, consuming them in the
/// process.
///
/// # Example
/// ```
/// use lambda_calculus::*;
///
/// assert_eq!(app(Var(1), Var(2)), App(Box::new((Var(1), Var(2)))));
/// ```
pub fn app(lhs: Term, rhs: Term) -> Term {
    App(Box::new((lhs, rhs)))
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_depth = self.max_depth();
        let max_free_index = self.max_free_index();
        let ctx = auto_generate_context(max_depth, max_free_index);
        let binder_names = generate_binder_names(&ctx, self.max_depth());
        write!(
            f,
            "{}",
            show_precedence_cla(&ctx, &binder_names, self, 0, 0)
        )
    }
}

/// A helper function to generate a default context for displaying a term.
fn auto_generate_context(max_depth: u32, max_free_index: usize) -> Context {
    let free_variables = (0..max_free_index)
        .map(|i| base26_encode(max_depth + i as u32))
        .collect::<Vec<_>>();
    free_variables.into()
}

/// A helper struct for displaying a `Term` with an external `Context`.
struct DisplayWithContext<'a> {
    term: &'a Term,
    ctx: &'a Context,
}

impl<'a> fmt::Display for DisplayWithContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let binder_names = generate_binder_names(self.ctx, self.term.max_depth());
        write!(
            f,
            "{}",
            show_precedence_cla(self.ctx, &binder_names, self.term, 0, 0)
        )
    }
}

/// Generates a list of fresh names for binders, avoiding clashes with the given context.
fn generate_binder_names(ctx: &Context, number: u32) -> Vec<String> {
    (0..)
        .map(|i| base26_encode(i as u32))
        .filter(|name| !ctx.contains(name))
        .take(number as usize)
        .collect()
}

fn base26_encode(mut n: u32) -> String {
    let mut buf = Vec::<u8>::new();
    n += 1;
    while n > 0 {
        let m = (n % 26) as u8;
        let m = if m == 0 { 26 } else { m };
        let c = m + b'a' - 1;
        buf.push(c);
        n = (n - 1) / 26
    }
    buf.reverse();
    String::from_utf8(buf).expect("error while printing term")
}

fn show_precedence_cla(
    ctx: &Context,
    binder_names: &[String],
    term: &Term,
    context_precedence: usize,
    depth: u32,
) -> String {
    match term {
        Var(0) => "undefined".to_owned(),
        Var(i) => {
            let i = *i as u32;
            if i <= depth {
                binder_names
                    .get((depth - i) as usize)
                    .expect("[BUG] binder_names are insufficient")
                    .to_owned()
            } else {
                let idx = (i - depth) as usize;
                ctx.resolve_free_var(idx)
                    .map_or(format!("<unknown{}>", idx), |s| s.to_owned())
            }
        }
        Abs(t) => {
            let ret = {
                format!(
                    "{}{}.{}",
                    LAMBDA,
                    binder_names
                        .get(depth as usize)
                        .expect("[BUG] binder_names are insufficient"),
                    show_precedence_cla(ctx, binder_names, t, 0, depth + 1)
                )
            };
            parenthesize_if(&ret, context_precedence > 1).into()
        }
        App(boxed) => {
            let (ref t1, ref t2) = **boxed;
            let ret = format!(
                "{} {}",
                show_precedence_cla(ctx, binder_names, t1, 2, depth),
                show_precedence_cla(ctx, binder_names, t2, 3, depth)
            );
            parenthesize_if(&ret, context_precedence == 3).into()
        }
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", show_precedence_dbr(self, 0))
    }
}

fn show_precedence_dbr(term: &Term, context_precedence: usize) -> String {
    match term {
        // Var(0) => "undefined".to_owned(),
        Var(i) => {
            format!("{:X}", i)
        }
        Abs(t) => {
            let ret = format!("{}{:?}", LAMBDA, t);
            parenthesize_if(&ret, context_precedence > 1).into()
        }
        App(boxed) => {
            let (ref t1, ref t2) = **boxed;
            let ret = format!(
                "{}{}",
                show_precedence_dbr(t1, 2),
                show_precedence_dbr(t2, 3)
            );
            parenthesize_if(&ret, context_precedence == 3).into()
        }
    }
}

fn parenthesize_if(input: &str, condition: bool) -> Cow<'_, str> {
    if condition {
        format!("({})", input).into()
    } else {
        input.into()
    }
}

/// A macro for chain application of `Term`s.
///
/// # Example
/// ```
/// # #[macro_use] extern crate lambda_calculus;
/// # fn main() {
/// use lambda_calculus::term::*;
///
/// assert_eq!(app!(Var(1), Var(2), Var(3)), app(app(Var(1), Var(2)), Var(3)));
/// # }
/// ```
#[macro_export]
macro_rules! app {
    ($term1:expr, $($term2:expr),+) => {
        {
            let mut term = $term1;
            $(term = app(term, $term2);)*
            term
        }
    };
}

/// A macro for multiple abstraction of `Term`s.
///
/// # Example
/// ```
/// # #[macro_use] extern crate lambda_calculus;
/// # fn main() {
/// use lambda_calculus::term::*;
///
/// assert_eq!(abs!(3, Var(1)), abs(abs(abs(Var(1)))));
/// # }
/// ```
#[macro_export]
macro_rules! abs {
    ($n:expr, $term:expr) => {{
        let mut term = $term;

        for _ in 0..$n {
            term = abs(term);
        }

        term
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn app_macro() {
        assert_eq!(
            app!(Var(4), app!(Var(1), Var(2), Var(3))),
            app(Var(4), app(app(Var(1), Var(2)), Var(3)))
        );
    }

    #[test]
    fn context_methods() {
        let ctx = Context::new(&["a", "b", "c"]);
        let empty_ctx = Context::empty();

        // len & is_empty
        assert_eq!(ctx.len(), 3);
        assert!(!ctx.is_empty());
        assert_eq!(empty_ctx.len(), 0);
        assert!(empty_ctx.is_empty());

        // contains
        assert!(ctx.contains("b"));
        assert!(!ctx.contains("d"));

        // iter
        let names: Vec<&str> = ctx.iter().collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn context_resolve_free_var() {
        let ctx = Context::new(&["a", "b", "c"]);

        // 1-based index, forward lookup
        assert_eq!(ctx.resolve_free_var(1), Some("a"));
        assert_eq!(ctx.resolve_free_var(3), Some("c"));

        // Invalid cases
        assert_eq!(ctx.resolve_free_var(0), None); // 0 is invalid
        assert_eq!(ctx.resolve_free_var(4), None); // Out of bounds
    }

    #[test]
    fn abs_macro() {
        assert_eq!(abs!(4, Var(1)), abs(abs(abs(abs(Var(1))))));

        assert_eq!(abs!(2, app(Var(1), Var(2))), abs(abs(app(Var(1), Var(2)))));
    }

    #[test]
    fn open_term_display() {
        assert_eq!(&abs(Var(2)).to_string(), "λa.b");
        assert_eq!(&abs(Var(3)).to_string(), "λa.c");
        assert_eq!(&abs!(2, Var(3)).to_string(), "λa.λb.c");
        assert_eq!(&abs!(2, Var(4)).to_string(), "λa.λb.d");
        assert_eq!(
            app!(
                Var(3),
                Var(4),
                abs(app(Var(4), Var(5))),
                abs!(2, app(Var(5), Var(6)))
            )
            .to_string(),
            "e f (λa.e f) (λa.λb.e f)"
        );
        assert_eq!(
            app!(
                abs!(2, app(Var(3), Var(4))),
                Var(1),
                Var(2),
                abs(app(Var(2), Var(3)))
            )
            .to_string(),
            "(λa.λb.c d) c d (λa.c d)"
        );
        assert_eq!(
            &app(abs(Var(1)), app(abs(app(Var(10), Var(1))), Var(10))).to_string(),
            "(λa.a) ((λa.j a) k)"
        );

        assert_eq!(
            abs!(
                27,
                app!(Var(28), Var(29), Var(30), Var(50), Var(702), Var(703))
            )
            .to_string(),
            "λa.λb.λc.λd.λe.λf.λg.λh.λi.λj.λk.λl.λm.λn.λo.λp.λq.λr.λs.λt.λu.λv.λw.λx.λy.λz.λaa.ab ac ad ax zz aaa"
        );
        assert_eq!(
            abs!(3, app!(Var(2), Var(3), Var(4))).to_string(),
            "λa.λb.λc.b a d"
        );
        assert_eq!(Var(26).to_string(), "z");
        assert_eq!(Var(27).to_string(), "aa");
    }

    #[test]
    fn display_modes() {
        let zero = abs!(2, Var(1));
        let succ = abs!(3, app(Var(2), app!(Var(3), Var(2), Var(1))));
        let pred = abs!(
            3,
            app!(
                Var(3),
                abs!(2, app(Var(1), app(Var(2), Var(4)))),
                abs(Var(2)),
                abs(Var(1))
            )
        );

        assert_eq!(&zero.to_string(), "λa.λb.b");
        assert_eq!(&succ.to_string(), "λa.λb.λc.b (a b c)");
        assert_eq!(
            &pred.to_string(),
            "λa.λb.λc.a (λd.λe.e (d b)) (λd.c) (λd.d)"
        );

        assert_eq!(&format!("{:?}", zero), "λλ1");
        assert_eq!(&format!("{:?}", succ), "λλλ2(321)");
        assert_eq!(&format!("{:?}", pred), "λλλ3(λλ1(24))(λ2)(λ1)");
    }

    #[test]
    fn term_display_with_context() {
        let ctx = Context::new(&["x", "y"]);

        // Term with only free variables: Var(1) -> x, Var(2) -> y
        let term1 = app(Var(1), Var(2));
        assert_eq!(term1.with_context(&ctx).to_string(), "x y");

        // Term with bound and free variables
        // λa. a y  (y is Var(2) from context)
        let term2 = abs(app(Var(1), Var(3)));
        assert_eq!(term2.with_context(&ctx).to_string(), "λa.a y");

        let term3 = abs(Var(2));
        assert_eq!(term3.with_context(&ctx).to_string(), "λa.x");
    }

    #[test]
    fn term_display_with_clashing_context() {
        let ctx = Context::new(&["a", "c"]);

        let term1 = app(Var(1), Var(2));
        assert_eq!(term1.with_context(&ctx).to_string(), "a c");

        let term2 = abs(app(Var(1), Var(3)));
        assert_eq!(term2.with_context(&ctx).to_string(), "λb.b c");

        let term3 = abs(Var(2));
        assert_eq!(term3.with_context(&ctx).to_string(), "λb.a");
    }

    #[test]
    fn term_display_without_context() {
        let term1 = app(Var(1), Var(2));
        assert_eq!(term1.to_string(), "a b");
        assert_eq!(
            term1.with_context(&Context::empty()).to_string(),
            "<unknown1> <unknown2>"
        );

        let term2 = abs(app(Var(1), Var(3)));
        assert_eq!(term2.to_string(), "λa.a c");
        assert_eq!(
            term2.with_context(&Context::empty()).to_string(),
            "λa.a <unknown2>"
        );

        let term3 = abs(Var(2));
        assert_eq!(term3.to_string(), "λa.b");
        assert_eq!(
            term3.with_context(&Context::empty()).to_string(),
            "λa.<unknown1>"
        );
    }

    #[test]
    fn is_supercombinator() {
        assert!(abs(Var(1)).is_supercombinator());
        assert!(app(abs(Var(1)), abs(Var(1))).is_supercombinator());
        assert!(abs!(10, Var(10)).is_supercombinator());
        assert!(abs!(10, app(Var(10), Var(10))).is_supercombinator());

        assert!(!Var(0).is_supercombinator());
        assert!(!Var(1).is_supercombinator());
        assert!(!abs(Var(2)).is_supercombinator());
        assert!(!app(abs(Var(1)), Var(1)).is_supercombinator());
        assert!(!abs!(10, Var(11)).is_supercombinator());
        assert!(!abs!(10, app(Var(10), Var(11))).is_supercombinator());
    }

    #[test]
    fn max_depth() {
        assert_eq!(Var(1).max_depth(), 0);
        assert_eq!(abs(Var(1)).max_depth(), 1);
        assert_eq!(abs!(10, Var(5)).max_depth(), 10);
        assert_eq!(
            app!(abs!(5, Var(2)), abs!(9, Var(4)), abs!(7, Var(6))).max_depth(),
            9
        );
    }

    #[test]
    fn is_isomorphic_to() {
        assert!(abs(Var(1)).is_isomorphic_to(&abs(Var(1))));
        assert!(!abs(Var(1)).is_isomorphic_to(&abs(Var(2))));
        assert!(!app(abs(Var(1)), Var(1)).is_isomorphic_to(&app(abs(Var(1)), Var(2))));
        assert!(app(abs(Var(1)), Var(1)).is_isomorphic_to(&app(abs(Var(1)), Var(1))));
        assert!(!app(abs(Var(1)), Var(1)).is_isomorphic_to(&app(Var(2), abs(Var(1)))));
    }

    #[test]
    fn has_free_variables() {
        assert!(!(abs(Var(1)).has_free_variables()));
        assert!(abs(Var(2)).has_free_variables());
        assert!(app(abs(Var(2)), Var(1)).has_free_variables());
        assert!(app(abs(Var(2)), abs(Var(1))).has_free_variables());
        assert!(app(abs(Var(1)), abs(Var(2))).has_free_variables());
        assert!(!app(abs(Var(1)), abs(Var(1))).has_free_variables());
        assert!(
            !(abs(app(
                abs(app(Var(2), app(Var(1), Var(1)))),
                abs(app(Var(2), app(Var(1), Var(1)))),
            )))
            .has_free_variables()
        );
        assert!((Var(0)).has_free_variables());
    }

    #[test]
    fn test_has_eta_redex() {
        // \x. f x has an eta redex
        let term = parse("λλ(λ0)0(λ1)", DeBruijn).unwrap();
        assert!(!term.has_eta_redex());
    }
}

#[cfg(test)]
mod eta_reduce_tests {
    use crate::*;

    #[test]
    fn test_no_reduction_on_plain_var() {
        // x
        let term = Var(0);
        assert_eq!(term.clone().eta_reduce(), term);
    }

    #[test]
    fn test_identity_does_not_reduce() {
        // \x. x -> Abs(Var(0))
        // Cannot be eta-reduced because it's not an application
        let term = abs(Var(0));
        assert_eq!(term.clone().eta_reduce(), term);
    }

    #[test]
    fn test_basic_eta_reduction() {
        // \x. f x  =>  Abs(App(Var(1), Var(0)))
        // Since `Var(1)` (which is `f`) doesn't contain `Var(0)`, this should reduce to `f`
        // But remember: when the Abs disappears, Var(1) shifts down to Var(0)
        let term = abs(app(Var(1), Var(0)));
        let expected = Var(0);
        assert_eq!(term.eta_reduce(), expected);
    }

    #[test]
    fn test_no_reduction_if_variable_is_free_in_m() {
        // \x. x x  =>  Abs(App(Var(0), Var(0)))
        // Here, the body M is `Var(0)`. Because count_bvar(0) == 1, it cannot be eta-reduced.
        let term = abs(app(Var(0), Var(0)));
        assert_eq!(term.clone().eta_reduce(), term);
    }

    #[test]
    fn test_nested_eta_reduction() {
        // \x. \y. f x y  => \x. \y. ((Var(2) Var(1)) Var(0))
        // Inner reduction: \y. (f x) y  => f x
        // Outer reduction: \x. f x      => f
        // Ultimately results in just `f`, shifted down properly to Var(0)
        let term = abs(abs(app(app(Var(2), Var(1)), Var(0))));
        let expected = Var(0);
        assert_eq!(term.eta_reduce(), expected);
    }

    #[test]
    fn test_complex_shifting() {
        // \x. (\y. g y) x
        // Inner term `\y. g y` can be eta-reduced to `g`
        // Let's build: Abs(App( Abs(App(Var(2), Var(0))), Var(0) ))
        // 1. Inner Abs reduces: Abs(App(Var(2), Var(0))) -> Var(1) (shifted)
        // 2. Tree becomes: Abs(App(Var(1), Var(0)))
        // 3. Outer Abs reduces: -> Var(0) (shifted again)
        let inner_abs = abs(app(Term::Var(2), Var(0))); // \y. g y
        let term = abs(app(inner_abs, Var(0))); // \x. (\y. g y) x

        let expected = Var(0);
        assert_eq!(term.eta_reduce(), expected);
    }
}
