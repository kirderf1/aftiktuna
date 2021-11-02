package me.kirderf.aftiktuna;

import javax.swing.*;
import java.awt.*;
import java.io.*;
import java.util.List;

public final class Main {
	
	public static void main(String[] args) throws IOException {
		System.out.println("Hello universe");
		System.out.println();
		
		boolean noGui = findFlag("--nogui", args);
		
		GameInstance instance;
		
		if (noGui) {
			instance = new GameInstance(new PrintWriter(System.out, true), new BufferedReader(new InputStreamReader(System.in)));
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
		PipedReader outReader = new PipedReader();
		PipedWriter inWriter = new PipedWriter();
		
		PrintWriter out = new PrintWriter(new PipedWriter(outReader), true);
		BufferedReader in = new BufferedReader(new PipedReader(inWriter));
		
		SwingUtilities.invokeLater(() -> {
			JFrame frame = new JFrame("Aftiktuna");
			
			frame.setDefaultCloseOperation(WindowConstants.EXIT_ON_CLOSE);
			
			frame.getContentPane().add(initOutputArea(new BufferedReader(outReader)), BorderLayout.NORTH);
			frame.getContentPane().add(initInputField(inWriter, out), BorderLayout.SOUTH);
			
			frame.pack();
			frame.setLocationRelativeTo(null);
			frame.setVisible(true);
		});
		
		return new GameInstance(out, in);
	}
	
	private static Component initOutputArea(BufferedReader reader) {
		
		JTextArea area = new JTextArea(15, 50);
		area.setEditable(false);
		
		new SwingWorker<Void, String>() {
			@Override
			protected Void doInBackground() throws Exception {
				while (!isCancelled()) {
					String next = reader.readLine();
					publish(next);
				}
				return null;
			}
			
			@Override
			protected void process(List<String> chunks) {
				if (!area.getText().isEmpty())
					area.append("\n");
				area.append(String.join("\n", chunks));
			}
		}.execute();
		
		JScrollPane scrollPane = new JScrollPane(area);
		scrollPane.setVerticalScrollBarPolicy(JScrollPane.VERTICAL_SCROLLBAR_ALWAYS);
		scrollPane.setHorizontalScrollBarPolicy(JScrollPane.HORIZONTAL_SCROLLBAR_NEVER);
		return scrollPane;
	}
	
	private static Component initInputField(Writer writer, PrintWriter out) {
		JTextField textField = new JTextField();
		textField.addActionListener(e -> {
			try {
				writer.write(textField.getText() + "\n");
				out.printf("> %s%n", textField.getText());
				textField.setText("");
			} catch(IOException ex) {
				ex.printStackTrace();
			}
		});
		
		return textField;
	}
}