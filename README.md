Solitaire
=========

A simple solitaire game using [rust](https://www.rust-lang.org) and [amethyst](https://github.com/amethyst/amethyst).

Running
-------

Before running the program you need to generate the card textures using the `gen.py` script in the `resourses` directory.
The gen.py script assumes that you have PIL installed, that your current directory is the resources directory, and that
`/usr/share/fonts/noto/NotoMono-Regular.ttf` is a valid ttf file. If this is not the case then it may be necessary to modify `gen.py`.

Under Arch Linux it is sufficient to install `noto-fonts` and `python-pillow`.

Once the cards have been generated, simply `cargo run`. The escape key will terminate the program.
