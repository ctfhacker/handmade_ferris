all: check build docs

build: check

check:
	cargo build --release || true

	# Clippy checks
	RUST_BACKTRACE=full cargo clippy --color always -- \
				   --allow clippy::verbose_bit_mask \
				   --allow clippy::print_with_newline \
				   --allow clippy::write_with_newline \
				   --deny  missing_docs \
				   --deny  clippy::missing_docs_in_private_items \
				   --deny  clippy::pedantic \
				   --allow clippy::struct_excessive_bools \
				   --allow clippy::redundant_field_names \
				   --allow clippy::must_use_candidate || true


docs: check
	# Documentation build regardless of arch
	cargo doc 
