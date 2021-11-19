package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.Item;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class TakeItemsCommand extends Command {
	private final Aftik aftik;
	
	public TakeItemsCommand(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public Status prepare() {
		return aftik.isAnyNearAccessible(Item.CAST.toPredicate(), true)
				? Status.KEEP : Status.REMOVE;
	}
	
	@Override
	public Status performAction(ActionPrinter out) {
		Optional<Item> optionalItem = aftik.findNearestAccessible(Item.CAST, true);
		
		if (optionalItem.isPresent()) {
			Item item = optionalItem.get();
			
			aftik.moveAndTake(item, out);
			
			return Status.KEEP;
		} else {
			out.printFor(aftik, "There are no nearby items to take.");
			return Status.REMOVE;
		}
	}
}