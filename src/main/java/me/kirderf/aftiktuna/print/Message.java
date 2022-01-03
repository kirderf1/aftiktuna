package me.kirderf.aftiktuna.print;

import java.util.Optional;

public interface Message {
	String getText();
	
	default Optional<Message> tryCombine(Message other) {
		return Optional.empty();
	}
}
