package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.print.ActionPrinter;

import java.io.PrintWriter;

public abstract class GameView {
	
	public abstract void printView(PrintWriter out);
	
	public abstract int handleInput(String input, PrintWriter out, ActionPrinter actionOut);
}