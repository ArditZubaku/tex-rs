fn main() {
    cc::Build::new()
        .file("csrc/termbox.c")
        .file("csrc/utf8.c")
        .include("csrc")
        .warnings(false)
        .compile("termbox");
}
