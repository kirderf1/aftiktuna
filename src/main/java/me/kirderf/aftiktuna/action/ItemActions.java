package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.*;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;

import java.util.Optional;

import static me.kirderf.aftiktuna.action.ActionHandler.argument;
import static me.kirderf.aftiktuna.action.ActionHandler.literal;

public final class ItemActions {
	static void register(CommandDispatcher<GameInstance> dispatcher) {
		dispatcher.register(literal("take").then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
				.executes(context -> takeItem(context.getSource(), ObjectArgument.getType(context, "item")))));
		dispatcher.register(literal("wield").then(argument("item", ObjectArgument.create(ObjectTypes.WEAPONS))
				.executes(context -> wieldItem(context.getSource(), ObjectArgument.getType(context, "item", WeaponType.class)))));
	}
	
	private static int takeItem(GameInstance game, ObjectType type) {
		Aftik aftik = game.getAftik();
		Optional<Item> optionalItem = aftik.findNearest(Item.CAST.filter(type::matching));
		if(optionalItem.isPresent()) {
			
			Item item = optionalItem.get();
			Optional<GameObject> blocking = aftik.findBlockingTo(item.getCoord());
			if (blocking.isEmpty()) {
				Optional<Entity.MoveFailure> move = aftik.moveAndTake(item);
				
				if(move.isEmpty()) {
					game.out().printf("You picked up the %s.%n", type.lowerCaseName());
				} else {
					ActionHandler.printMoveFailure(game, move.get());
				}
				
				return 1;
			} else {
				ActionHandler.printBlocking(game, blocking.get());
				return 0;
			}
		} else {
			game.out().printf("There is no %s here to pick up.%n", type.lowerCaseName());
			return 0;
		}
	}
	
	private static int wieldItem(GameInstance game, WeaponType itemType) {
		Aftik aftik = game.getAftik();
		
		if (aftik.wieldFromInventory(itemType)) {
			game.out().printf("You wielded a %s.%n", itemType.lowerCaseName());
			return 1;
		} else {
			Optional<Item> optionalItem = aftik.findNearest(Item.CAST.filter(itemType::matching));
			if(optionalItem.isPresent()) {
				
				Item item = optionalItem.get();
				Optional<GameObject> blocking = aftik.findBlockingTo(item.getCoord());
				if (blocking.isEmpty()) {
					Optional<Entity.MoveFailure> move = aftik.moveAndWield(item, itemType);
					if(move.isEmpty()) {
						game.out().printf("You picked up and wielded the %s.%n", itemType.lowerCaseName());
					} else {
						ActionHandler.printMoveFailure(game, move.get());
					}
					return 1;
				} else {
					ActionHandler.printBlocking(game, blocking.get());
					return 0;
				}
			} else {
				game.out().printf("There is no %s that you can wield.%n", itemType.lowerCaseName());
				return 0;
			}
		}
	}
}
