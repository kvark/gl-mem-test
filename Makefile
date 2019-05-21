.PHONY: all

all:
	cargo build
	target/debug/gl-mem-test 2d image false
	target/debug/gl-mem-test 2d image true
	target/debug/gl-mem-test 2d storage
	target/debug/gl-mem-test rect image
	target/debug/gl-mem-test rect storage
