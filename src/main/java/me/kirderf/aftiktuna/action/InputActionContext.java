package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.function.Consumer;

public final class InputActionContext {
	private final ActionPrinter out;
	private final GameInstance game;
	private boolean showView = false;
	private boolean isUsed = false;
	
	public InputActionContext(GameInstance game, ActionPrinter out) {
		this.game = game;
		this.out = out;
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
	
	public boolean shouldShowView() {
		return showView;
	}
	
	public int printNoAction(String text, Object... args) {
		return noAction(out -> out.print(text, args));
	}
	
	public int noAction(Consumer<ActionPrinter> messages) {
		onUse();
		messages.accept(out);
		return 0;
	}
	
	public int noActionWithView(Consumer<ActionPrinter> messages) {
		showView = true;
		return noAction(messages);
	}
	
	public int action() {
		onUse();
		return 1;
	}
	
	public int action(Consumer<ActionPrinter> action) {
		onUse();
		action.accept(out);
		return 1;
	}
	
	private void onUse() {
		if (isUsed)
			throw new IllegalStateException("This context has already been used.");
		isUsed = true;
	}
}
