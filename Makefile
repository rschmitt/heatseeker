default: build

clean:
	$(RM) selectric

build: clean
	rustc -o selectric selectric.rs

test:
	rustc -A dead_code -A unstable --test selectric.rs
	./selectric
