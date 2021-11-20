package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.action.InputActionContext;

import java.io.PrintWriter;

public abstract class GameView {
	
	public abstract void printView(PrintWriter out);
	
	public abstract int handleInput(String input, InputActionContext context);
}