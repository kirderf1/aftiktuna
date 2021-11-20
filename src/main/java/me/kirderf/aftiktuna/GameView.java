package me.kirderf.aftiktuna;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.action.InputActionContext;

import java.io.PrintWriter;

public abstract class GameView {
	
	public abstract int handleInput(String input, InputActionContext context) throws CommandSyntaxException;
	
	public abstract void printView(PrintWriter out);
}