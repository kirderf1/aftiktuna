package me.kirderf.aftiktuna;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.action.InputActionContext;
import me.kirderf.aftiktuna.action.StoreCommands;
import me.kirderf.aftiktuna.object.ItemType;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;

import java.io.PrintWriter;
import java.util.List;

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
		List<ItemType> stock = shopkeeper.getItemsInStock();
		
		int maxLength = stock.stream().map(item -> item.capitalizedName().length()).max(Integer::compareTo).orElse(0);
		for (ItemType item : shopkeeper.getItemsInStock()) {
			String name = item.capitalizedName();
			
			out.printf("%s | %dp%n", name + " ".repeat(maxLength - name.length()), item.getPrice());
		}
	}
}