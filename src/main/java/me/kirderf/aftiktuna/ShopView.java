package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.io.PrintWriter;

public final class ShopView extends GameView {
	private final GameInstance game;
	private final Shopkeeper shopkeeper;
	
	public ShopView(GameInstance game, Shopkeeper shopkeeper) {
		this.game = game;
		this.shopkeeper = shopkeeper;
	}
	
	@Override
	public void printView(PrintWriter out) {
		out.println("Fuel Can | 7000p");
	}
	
	@Override
	public int handleInput(String input, PrintWriter out, ActionPrinter actionOut) {
		return StoreCommands.handleInput(input, new StoreCommands.StoreContext(game, shopkeeper, out, actionOut));
	}
}