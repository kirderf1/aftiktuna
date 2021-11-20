package me.kirderf.aftiktuna;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.action.InputActionContext;
import me.kirderf.aftiktuna.object.ItemType;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;

import java.io.PrintWriter;

public final class ShopView extends GameView {
	private final Shopkeeper shopkeeper;
	
	public ShopView(Shopkeeper shopkeeper) {
		this.shopkeeper = shopkeeper;
	}
	
	@Override
	public int handleInput(String input, InputActionContext context) throws CommandSyntaxException {
		return StoreCommands.handleInput(input, new StoreCommands.StoreContext(context, shopkeeper));
	}
	
	@Override
	public void printView(PrintWriter out) {
		for (ItemType item : shopkeeper.getItemsInStock()) {
			out.printf("%s | %dp%n", item.capitalizedName(), item.getPrice());
		}
	}
}