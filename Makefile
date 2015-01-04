default: build

clean:
	$(RM) selectric

build: clean
	rustc -A dead_code -o selectric selectric.rs

test:
	rustc -A dead_code --test selectric.rs
	./selectric
