package me.kirderf.aftiktuna.print;

import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Collects action messages to be flushed to the view.
 */
public final class MessageBuffer implements SimplePrinter {
	private final List<Message> messages = new ArrayList<>();
	
	@Override
	public void println() {
		print("");
	}
	
	@Override
	public void print(String message, Object... args) {
		messages.add(() -> message.formatted(args));
	}
	
	@Override
	public void print(Message message) {
		if (!messages.isEmpty()) {
			Message prev = messages.get(messages.size() - 1);
			Optional<Message> optionalCombined = prev.tryCombine(message);
			if (optionalCombined.isPresent()) {
				messages.remove(messages.size() - 1);
				messages.add(optionalCombined.get());
			} else
				messages.add(message);
		} else
			messages.add(message);
	}
	
	public void flush(PrintWriter out) {
		for (Message message : messages) {
			out.println(message.getText());
		}
		messages.clear();
	}
}