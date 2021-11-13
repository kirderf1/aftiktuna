package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.StringArgumentType;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.object.*;
import me.kirderf.aftiktuna.object.entity.Aftik;

import java.util.Optional;

import static me.kirderf.aftiktuna.action.ActionHandler.argument;
import static me.kirderf.aftiktuna.action.ActionHandler.literal;

public final class ItemActions {
	static void register(CommandDispatcher<GameInstance> dispatcher) {
		dispatcher.register(literal("take").then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
				.executes(context -> takeItem(context.getSource(), ObjectArgument.getType(context, "item")))));
		dispatcher.register(literal("wield").then(argument("item", ObjectArgument.create(ObjectTypes.WEAPONS))
				.executes(context -> wieldItem(context.getSource(), ObjectArgument.getType(context, "item", WeaponType.class)))));
		dispatcher.register(literal("give").then(argument("name", StringArgumentType.string())
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS)).executes(context -> giveItem(context.getSource(),
						StringArgumentType.getString(context, "name"), ObjectArgument.getType(context, "item"))))));
	}
	
	private static int takeItem(GameInstance game, ObjectType type) {
		Aftik aftik = game.getAftik();
		return ActionHandler.searchForAndIfNotBlocked(game, aftik, Item.CAST.filter(type::matching),
				item -> aftik.moveAndTake(item, game.out()),
				() -> game.directOut().printf("There is no %s here to pick up.%n", type.name()));
	}
	
	private static int wieldItem(GameInstance game, WeaponType weaponType) {
		Aftik aftik = game.getAftik();
		
		if (aftik.hasItem(weaponType)) {
			aftik.wieldFromInventory(weaponType, game.out());
			return 1;
		} else {
			return ActionHandler.searchForAndIfNotBlocked(game, aftik, Item.CAST.filter(weaponType::matching),
					item -> aftik.moveAndWield(item, weaponType, game.out()),
					() -> game.directOut().printf("There is no %s that %s can wield.%n", weaponType.name(), aftik.getName()));
		}
	}
	
	private static int giveItem(GameInstance game, String name, ObjectType type) {
		Aftik aftik = game.getAftik();
		Optional<Aftik> aftikOptional = game.getCrew().findByName(name);
		
		if (aftikOptional.isPresent() && aftik.getRoom() == aftikOptional.get().getRoom()) {
			Aftik target = aftikOptional.get();
			
			if (aftik == target) {
				game.directOut().printf("%s can't give an item to themselves.%n", aftik.getName());
				return 0;
			}
			
			if (aftik.hasItem(type)) {
				
				return ActionHandler.ifNotBlocked(game, aftik, target, () -> aftik.moveAndGive(target, type, game.out()));
			} else {
				game.directOut().printf("%s does not have that item.%n", aftik.getName());
				return 0;
			}
		} else {
			game.directOut().printf("There is no such aftik in the room.%n");
			return 0;
		}
	}
}