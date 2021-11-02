package me.kirderf.aftiktuna;

import javax.swing.*;
import java.awt.*;
import java.io.*;

public final class Main {
	
	public static void main(String[] args) throws IOException {
		System.out.println("Hello universe");
		
		boolean noGui = findFlag("--nogui", args);
		
		GameInstance instance;
		
		if (noGui) {
			instance = new GameInstance(new BufferedReader(new InputStreamReader(System.in)));
		} else {
			instance = initGuiGame();
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
	
	private static GameInstance initGuiGame() throws IOException {
		PipedReader in = new PipedReader();
		PipedWriter inWriter = new PipedWriter(in);
		
		SwingUtilities.invokeLater(() -> {
			JFrame frame = new JFrame("Aftiktuna");
			
			frame.setDefaultCloseOperation(WindowConstants.EXIT_ON_CLOSE);
			
			frame.getContentPane().add(initInputField(inWriter));
			
			frame.pack();
			frame.setLocationRelativeTo(null);
			frame.setVisible(true);
		});
		
		return new GameInstance(new BufferedReader(in));
	}
	
	private static Component initInputField(Writer writer) {
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
		
		return textField;
	}
}