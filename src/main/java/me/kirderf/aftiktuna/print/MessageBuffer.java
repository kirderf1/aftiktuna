package me.kirderf.aftiktuna.print;

import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.List;

/**
 * Collects action messages to be flushed to the view.
 */
public final class MessageBuffer {
	private final List<String> messages = new ArrayList<>();
	
	public void println() {
		messages.add("");
	}
	
	public void print(String message, Object... args) {
		messages.add(message.formatted(args));
	}
	
	public void flush(PrintWriter out) {
		for (String message : messages) {
			out.println(message);
		}
		messages.clear();
	}
}