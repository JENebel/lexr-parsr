#[macro_export]
/// Define a lexer function with provided rules.
///
/// The lexer function takes a string slice and returns a vector of tokens and their locations.
///
/// If it is unable to parse an input, it returns an error with the first character in the unmatched subsequence, and the location of the error.
///
/// Usage example:
///
///     /*use parcom::lexer;
///
///     #[derive(Debug, PartialEq)]
///     pub enum Token {
///         Word(String),
///         Number(u32),
///         EndOfFile,
///     }
///
///     // Statics and constants can be used to reuse regexes
///     const WORD: &str = r"[a-zA-Z]+";
/// 
///     lexer!{lex -> Token {
///         r"\s+" =>         |_|  continue, // Ignore whitespace. 'continue' is the only allowed expression except for tokens and panic
///         "[0-9]+" =>       |i|  Token::Number(i.parse().unwrap()),
///         WORD =>           |id| { // You can use blocks
///                                    println!("{}", id); 
///                                    Token::Word(id.to_string()) },
///         "#" WORD "#" =>   |_|  continue, // You can use a sequence of regexes
///         "$" =>            |_|  Token::EndOfFile
///     }}
///
///     assert!(lex("123 abc #comment#").unwrap().into_iter().map(|(t, _)| t).collect::<Vec<_>>() == vec![
///         Token::Number(123), 
///         Token::Word("abc".to_string()), 
///         Token::EndOfFile
///     ]);*/
/// 
macro_rules! lexer {
    ($v:vis $name:ident $(($($arg:ident: $arg_typ:ty),*))? -> $token:ty {$($regpat:tt $($regex:expr)* => |$id:pat_param| $closure:expr),* $(,)?}) => {
    concat_idents::concat_idents!(name = _LEXER_, $name {
        #[allow(unused_imports)]
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        pub struct name<'a> {
            input: &'a str,
            input_iter: std::str::Chars<'a>,
            idx: usize,
            line: usize,
            col: usize,
            empty: bool,
            $($($arg: $arg_typ),*)?
        }

        impl<'a> name<'a> {
            pub fn new(input: &'a str $(,$($arg: $arg_typ),*)?) -> Self {
                name {
                    input,
                    input_iter: input.chars(),
                    idx: 0,
                    line: 1,
                    col: 1,
                    empty: false,
                    $($($arg),*)?
                }
            }
        }

        impl<'a> Iterator for name<'a> {
            type Item = ($token, parcom::SrcLoc);

            #[allow(unreachable_code)]
            fn next(&mut self) -> Option<Self::Item> {
                $($(let $arg: $arg_typ = self.$arg);*)?;

                // This here to allow for matching $ as eof a single time while avoiding infinite loop
                if self.empty { return None }
                if self.idx == self.input.len() { self.empty = true; }

                loop {
                    $(
                        let re = lexer!(@regex_rule $regpat $($regex)*);
                        if let Some(mat) = re.find(&self.input[self.idx..]) {
                            let length = mat.end();
                            let $id = mat.as_str();
                            
                            let start = (self.line, self.col);
                            let mut end = start;
                        
                            for i in 0..length {
                                let c = self.input_iter.next().unwrap();
                                if i == length - 1 {
                                    end = (self.line, self.col);
                                }
                                if c == '\n' {
                                    self.line += 1;
                                    self.col = 1;
                                } else {
                                    self.col += 1;
                                }
                            }

                            self.idx += length;

                            let token = $closure; // If the closure is a continue, it skips the push
                            return Some((token, parcom::SrcLoc::new(start, end)));
                        }
                    )*
                }

                if let Some(c) = self.input_iter.next() {
                    panic!("Unexpected character '{}' at {}:{}:{}", c, self.line, self.col, self.idx);
                }

                None
            }
        }
        /// The lexer function
        /// 
        /// Returns a vector of tokens and their locations
        pub fn $name(input: &str $(,$($arg: $arg_typ),*)?) -> name {
            name::new(input $(,$($arg),*)?)
        };
    })};

    (@regex_rule _) => {
        {
            lazy_static::lazy_static! {
                static ref REGEX: regex::Regex = regex::Regex::new("(?s).").unwrap();
            }; 
            &REGEX
        }
    };

    (@regex_rule $($regex:expr)+) => {
        {
            lazy_static::lazy_static! {
                static ref REGEX: regex::Regex = regex::Regex::new({
                    let mut r_str = "^".to_string();
                    $(r_str.push_str($regex);)+
                    r_str
                }.as_str()).unwrap();
            }; 
            &REGEX
        }
    }
}