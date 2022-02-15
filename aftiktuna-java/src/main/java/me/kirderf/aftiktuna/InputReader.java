package me.kirderf.aftiktuna;

import java.io.BufferedReader;
import java.io.IOException;

public final class InputReader {
	private final BufferedReader in;
	private final Runnable prepareForInput;
	
	public InputReader(BufferedReader in, Runnable prepareForInput) {
		this.in = in;
		this.prepareForInput = prepareForInput;
	}
	
	public String readLine() {
		try {
			prepareForInput.run();
			return in.readLine();
		} catch(IOException e) {
			throw new RuntimeException(e);
		}
	}
}