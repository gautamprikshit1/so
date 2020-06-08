#[macro_export]
macro_rules! print_error {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Red);
        termimad::mad_write_inline!(err, $skin, "✖ ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_notice {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Yellow);
        termimad::mad_write_inline!(err, $skin, "➜ ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_success {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Green);
        termimad::mad_write_inline!(err, $skin, "✔ ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_log {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Blue);
        termimad::mad_write_inline!(err, $skin, "• ").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}

#[macro_export]
macro_rules! print_warn {
    ($skin: expr, $md: literal $(, $value: expr )* $(,)? ) => {{
        use lazy_static::lazy_static;
        use minimad::mad_inline;
        use crate::error::Error;
        let err = &mut std::io::stderr();
        let p = $skin.paragraph.clone();
        $skin.paragraph.set_fg(crossterm::style::Color::Magenta);
        termimad::mad_write_inline!(err, $skin, "⚡").map_err(Error::from)?;
        $skin.write_composite(err, mad_inline!($md $(, $value)*)).map_err(Error::from)?;
        $skin.paragraph = p;
        Ok::<(), Error>(())
    }};
}
