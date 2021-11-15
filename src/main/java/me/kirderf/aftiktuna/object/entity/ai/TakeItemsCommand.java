package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.Item;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

import java.util.Optional;

public final class TakeItemsCommand extends Command {
	private final Aftik aftik;
	
	public TakeItemsCommand(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		Optional<Item> optionalItem = aftik.findNearestAccessible(Item.CAST, true);
		
		if (optionalItem.isPresent()) {
			Item item = optionalItem.get();
			
			aftik.moveAndTake(item, out);
			
			return aftik.findNearestAccessible(Item.CAST, true).isEmpty();
		} else {
			out.printFor(aftik, "There are no nearby items to take.%n");
			return true;
		}
	}
}