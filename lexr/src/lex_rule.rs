#[doc(hidden)]
pub use concat_idents::concat_idents;
pub use lazy_static;
pub use regex;

#[macro_export]
/// Define a lexer function with provided rules.
///
/// The lexer function takes a string slice and returns a vector of tokens and their locations.
///
/// If it is unable to parse an input, it returns an error with the first character in the unmatched subsequence, and the location of the error.
/// 
/// More documentation can be found in the [crate root](crate).
///
/// # Examples
///
///     use lexr::lex_rule;
///
///     #[derive(PartialEq, Debug)]
///     pub enum Token {
///         Word(String),
///         Number(u32),
///         EndOfFile,
///     }
///
///     // Statics and constants can be used to reuse regexes
///     const WORD: &str = r"[a-zA-Z]+";
/// 
///     lex_rule!{lex -> Token {
///         r"\s+" =>         |_|  continue, // Ignore whitespace. 'continue' is the only allowed expression except for tokens and panic
///         "[0-9]+" =>       |i|  Token::Number(i.parse().unwrap()),
///         WORD =>           |id| { println!("{}", id); Token::Word(id.to_string()) },
///         "#" WORD "#" =>   |_|  continue, // You can use a sequence of regexes
///         eof =>            |_|  Token::EndOfFile
///     }}
///
///     let result = lex("123 abc #comment#").into_token_vec();
///     assert_eq!(result, vec![
///         Token::Number(123), 
///         Token::Word("abc".to_string()), 
///         Token::EndOfFile
///     ]);
///
macro_rules! lex_rule {
    ($v:vis $name:ident $(<$($lt:lifetime),+>)? $(($($arg:ident: $arg_typ:ty),*))? -> $token:ty {
        $($regpat:tt $($regex:expr)* => |$id:pat_param $(,$src_id:pat_param $(,$loc_id:pat_param)?)?| $closure:expr),* $(,)?
    }) => {
    lexr::concat_idents!(name = _LEXER_, $name {
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        /// Automatically generated lexer struct. Do not access its fields directly!
        /// 
        /// The `tokens` method returns an iterator over the tokens, stripping away the source locations.
        /// 
        /// `vec` and `token_vec` methods are provided for convenience.
        $v struct name<'_buf, $($($lt),+)?> {
            buf: lexr::LexBuf<'_buf>,
            $($($arg: $arg_typ),*)?
        }

        impl<'_buf $(,$($lt),+)?> From<name<'_buf, $($($lt),+)?>> for lexr::Lexer<$token, name<'_buf $(,$($lt),+)?>> {
            fn from(lexer: name<'_buf $(,$($lt),+)?>) -> Self {
                lexr::Lexer::new(lexer)
            }
        }

        impl<'_src, $($($lt),+)?> Iterator for name<'_src, $($($lt),+)?> {
            type Item = ($token, lexr::SrcLoc);

            #[allow(unreachable_code)]
            fn next(&mut self) -> Option<Self::Item> {
                $($(let $arg: $arg_typ = self.$arg);*)?;

                let start_idx = *self.buf.idx.borrow();

                let mut matched = false;
                loop {
                    // These allow for seamless matching of eof
                    matched = false;
                    let mut src = self.buf.source.borrow_mut();
                    if *self.buf.empty.borrow() { break }
                    if src.len() == 0 { *self.buf.empty.borrow_mut() = true; }
                    
                    $(
                    let regex = lex_rule!(@regex_rule $regpat $($regex)*);
                    if let Some(mat) = regex.find(&src) {
                        matched = true;
                        let length = mat.end();
                        
                        let start = (*self.buf.line.borrow(), *self.buf.col.borrow());
                        let mut end = start;
                        
                        let mut source_iter = src.chars();
                        for i in 0..length {
                            let c = source_iter.next().unwrap();
                            if i == length - 1 {
                                end = (*self.buf.line.borrow(), *self.buf.col.borrow());
                            }
                            if c == '\n' {
                                *self.buf.line.borrow_mut() += 1;
                                *self.buf.col.borrow_mut() = 1;
                            } else {
                                *self.buf.col.borrow_mut() += 1;
                            }
                        }

                        *src = &src[length..];
                        let end_idx = start_idx + length;
                        self.buf.idx.replace(end_idx);

                        let $id = mat.as_str();
                        $($(let $loc_id = lexr::SrcLoc::new(start, end, (start_idx, end_idx));)?)?
                        drop(src);
                        let token = {
                            $(let $src_id = self.buf.share();)?
                            $closure
                        };

                        return Some((token, lexr::SrcLoc::new(start, end, (start_idx, end_idx))));
                    })*

                    break
                }

                if !*self.buf.empty.borrow() && !matched {
                    if let Some(c) = self.buf.source.borrow().chars().next() {
                        panic!("Unexpected character '{}' at {}", c, lexr::SrcLoc::new((*self.buf.line.borrow(), *self.buf.col.borrow()), (*self.buf.line.borrow(), *self.buf.col.borrow()), (*self.buf.idx.borrow(), *self.buf.idx.borrow())));
                    }
                }

                None
            }
        }

        #[doc(hidden)]
        #[must_use]
        /// Creates a new lexer from a string slice.
        /// 
        /// A [`Lexer`](crate::Lexer) is returned, which can be used to iterate over the tokens.
        $v fn $name<'_buf $(,$($lt),+)?>(buf: impl Into<lexr::LexBuf<'_buf>> $(,$($arg: $arg_typ),*)?) -> lexr::Lexer<$token, name<'_buf $(,$($lt),+)?>> {
            lexr::Lexer::new(name {
                buf: buf.into(),
                $($($arg),*)?
            })
        }
    });};

    (@regex_rule _) => {{
        lexr::lazy_static::lazy_static! {
            static ref REGEX: lexr::regex::Regex = lexr::regex::Regex::new(r"(?s)^.").unwrap();
        }; 
        &REGEX
    }};

    (@regex_rule eof) => {{
        lexr::lazy_static::lazy_static!{
            static ref REGEX: lexr::regex::Regex = lexr::regex::Regex::new(r"^\z").unwrap();
        }; 
        &REGEX
    }};

    (@regex_rule ws) => {{
        lexr::lazy_static::lazy_static!{
            static ref REGEX: lexr::regex::Regex = lexr::regex::Regex::new(r"^[ \n\r\t]").unwrap();
        }; 
        &REGEX
    }};

    (@regex_rule $($regex:expr)+) => {{
        lexr::lazy_static::lazy_static!{
            static ref REGEX: lexr::regex::Regex = lexr::regex::Regex::new({
                let mut r_str = "^".to_string();
                $(r_str.push_str($regex);)+
                r_str
            }.as_str()).unwrap();
        }; 
        &REGEX
    }};
}