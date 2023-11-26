trace=trace.dat

run:
	@ - rm test.txt reference.txt
	@cargo build --release
	@echo "Running mine..."
	@./target/release/tomasulos < $(trace) > test.txt
	@echo "Running reference..."
	@ ./dynamsched < $(trace) > reference.txt
	
	@echo `diff test.txt reference.txt | grep "<" | wc -l` lines differ in outputs