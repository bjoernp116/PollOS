_default:
  @just --choose

build:
  @cargo build && cargo bootimage

run display="sdl": build
  qemu-system-x86_64 \
    -drive \
        format=raw,\
        file=target/x86_64-pollos/debug/bootimage-pollos.bin \
    -display {{display}} \
    -serial stdio \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -vga std

test:
    @cargo test

edit:
    nvim ./justfile
