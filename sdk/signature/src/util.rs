pub fn fmt_hex_string(f: &mut std::fmt::Formatter, data: &[u8]) -> std::fmt::Result {
    f.write_str("0x")?;
    data.iter()
        .try_for_each(|byte| f.write_fmt(format_args!("{:x?}", byte)))
}
