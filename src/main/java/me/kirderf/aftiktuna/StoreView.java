package me.kirderf.aftiktuna;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.store.StoreCommands;
import me.kirderf.aftiktuna.object.ItemType;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.print.StatusPrinter;

import java.io.PrintWriter;
import java.util.List;

public final class StoreView extends GameView {
	private final StatusPrinter statusPrinter;
	private final Shopkeeper shopkeeper;
	
	public StoreView(StatusPrinter statusPrinter, Shopkeeper shopkeeper) {
		this.statusPrinter = statusPrinter;
		this.shopkeeper = shopkeeper;
	}
	
	@Override
	public int handleInput(String input, CommandContext context) throws CommandSyntaxException {
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
		
		statusPrinter.printCrewPoints(true);
	}
}