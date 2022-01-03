package me.kirderf.aftiktuna.print;

public interface SimplePrinter {
	void println();
	
	void print(String message, Object... args);
}