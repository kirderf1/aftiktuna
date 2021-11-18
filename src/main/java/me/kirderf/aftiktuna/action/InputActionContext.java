package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

import java.io.PrintWriter;
import java.util.function.Consumer;

public final class InputActionContext {
	private final PrintWriter out;
	private final ContextPrinter actionOut;
	private final GameInstance game;
	
	public InputActionContext(GameInstance game, PrintWriter out, ContextPrinter actionOut) {
		this.game = game;
		this.out = out;
		this.actionOut = actionOut;
	}
	
	public Aftik getControlledAftik() {
		return game.getCrew().getAftik();
	}
	
	public GameInstance getGame() {
		return game;
	}
	
	public Crew getCrew() {
		return game.getCrew();
	}
	
	public int printNoAction(String text, Object... args) {
		return noAction(out -> out.printf(text, args));
	}
	
	public int noAction(Consumer<PrintWriter> messages) {
		messages.accept(out);
		return 0;
	}
	
	public int action() {
		return 1;
	}
	
	public int action(Consumer<ContextPrinter> action) {
		action.accept(actionOut);
		return 1;
	}
}
