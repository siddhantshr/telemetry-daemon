CC = clang
RS = cargo
CFLAGS = -Wall -Wextra -O2

BUILD_DIR = daemon/build
RUST_SRC := $(shell find src -name "*.rs")
DATA_DIR = /var/lib/teld

VENV = .venv
PYTHON = $(VENV)/bin/python
PIP = $(VENV)/bin/pip

.PHONY: all run clean install uninstall

all: launcher target/release/teld-worker | $(DATA_DIR)

$(VENV)/bin/activate: requirements.txt
	python3 -m venv $(VENV)
	$(PIP) install -r requirements.txt

visualizer: $(VENV)/bin/activate data-analyzer/__main__.py | $(DATA_DIR)
	@echo "Running visualizer..."
	$(PYTHON) data-analyzer/__main__.py

install: target/release/teld-worker
	sudo cp target/release/teld-worker /usr/local/bin/teld-worker

target/release/teld-worker: $(RUST_SRC) Cargo.toml
	@echo "Building teld-worker..."
	$(RS) build --release

launcher: daemon/launcher.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -o $(BUILD_DIR)/launcher $<

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(DATA_DIR):
	@echo "Creating data directory at $(DATA_DIR)..."
	sudo mkdir -p $(DATA_DIR)
	sudo chown $(USER):$(USER) $(DATA_DIR)

run: all install
	./$(BUILD_DIR)/launcher

kill:
	@echo "Stopping teld-worker..."
	kill $(shell cat /var/lib/teld/teld-worker.pid) || echo "teld-worker not running"

clean:
	$(RS) clean
	rm -rf $(BUILD_DIR)
	rm -f /tmp/teld-worker.log

clean-data:
	sudo rm -rf $(DATA_DIR)

uninstall:
	sudo rm -rf $(DATA_DIR)
	sudo rm -f /usr/local/bin/teld-worker