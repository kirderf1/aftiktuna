package me.kirderf.aftiktuna;

import javax.swing.*;
import java.awt.*;
import java.io.*;
import java.util.function.Consumer;

public final class Main {
	
	public static void main(String[] args) throws IOException {
		System.out.println("Hello universe");
		
		boolean noGui = findFlag("--nogui", args);
		
		GameInstance instance;
		
		if (noGui) {
			instance = new GameInstance(new BufferedReader(new InputStreamReader(System.in)));
		} else {
			instance = createWindow();
		}
		instance.run();
	}
	
	private static boolean findFlag(String flag, String[] args) {
		for (String arg : args) {
			if (flag.equals(arg))
				return true;
		}
		return false;
	}
	
	private static GameInstance createWindow() throws IOException {
		JFrame frame = new JFrame("Aftiktuna");
		
		frame.setDefaultCloseOperation(WindowConstants.EXIT_ON_CLOSE);
		
		Reader in = setupInputField(frame.getContentPane()::add);
		
		frame.pack();
		frame.setLocationRelativeTo(null);
		frame.setVisible(true);
		
		return new GameInstance(new BufferedReader(in));
	}
	
	private static Reader setupInputField(Consumer<Component> componentConsumer) throws IOException {
		PipedWriter writer = new PipedWriter();
		JTextField textField = new JTextField();
		textField.addActionListener(e -> {
			try {
				writer.write(textField.getText() + "\n");
				System.out.printf("> %s%n", textField.getText());
				textField.setText("");
			} catch(IOException ex) {
				ex.printStackTrace();
			}
		});
		componentConsumer.accept(textField);
		return new PipedReader(writer);
	}
}