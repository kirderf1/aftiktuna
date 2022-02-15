package me.kirderf.aftiktuna.command;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.print.SimplePrinter;

import java.util.Optional;
import java.util.function.Consumer;

/**
 * Context for handling user commands. A user command should either generate a user action (with action())
 * or a non-action (with a function with "noAction" in the name).
 * A user action will let the game tick proceed, while a non-action will immediately go back to take another user command.
 * Non-action is usually a preparation failure where the user command does not work in the current context, but this is not strictly the case.
 */
public final class CommandContext {
	private final SimplePrinter out;
	private final GameInstance game;
	private final CommandState state;
	private boolean showView = false;
	private boolean isUsed = false;
	private Consumer<ActionPrinter> action;
	
	public CommandContext(GameInstance game, CommandState state, SimplePrinter out) {
		this.game = game;
		this.state = state;
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
	
	public Optional<Area> getPreviousArea() {
		return game.lookupArea(state.getPreviousArea());
	}
	
	public boolean shouldShowView() {
		return showView;
	}
	
	public Optional<Consumer<ActionPrinter>> getAction() {
		return Optional.of(action);
	}
	
	public int printNoAction(String text, Object... args) {
		return noAction(out -> out.print(text, args));
	}
	
	public int noAction(Consumer<SimplePrinter> messages) {
		onUse();
		messages.accept(out);
		return 0;
	}
	
	public int noActionWithView(Consumer<SimplePrinter> messages) {
		showView = true;
		return noAction(messages);
	}
	
	public int action() {
		return action(out -> {});
	}
	
	public int action(Consumer<ActionPrinter> action) {
		onUse();
		this.action = action;
		return 1;
	}
	
	private void onUse() {
		if (isUsed)
			throw new IllegalStateException("This context has already been used.");
		isUsed = true;
	}
}
