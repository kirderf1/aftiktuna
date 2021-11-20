package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.action.InputActionContext;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.print.StatusPrinter;

import java.io.PrintWriter;

public final class AreaView extends GameView {
	private final GameInstance game;
	
	public AreaView(GameInstance game) {
		this.game = game;
	}
	
	@Override
	public void printView(PrintWriter out) {
		StatusPrinter.printArea(game.getCrew().getAftik().getArea(), out);
	}
	
	@Override
	public int handleInput(String input, PrintWriter out, ActionPrinter actionOut) {
		return ActionHandler.handleInput(new InputActionContext(game, out, actionOut), input);
	}
}