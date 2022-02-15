package me.kirderf.aftiktuna;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.command.CommandContext;

import java.io.PrintWriter;

public abstract class GameView {
	
	public abstract int handleInput(String input, CommandContext context) throws CommandSyntaxException;
	
	public abstract void printView(PrintWriter out);
}