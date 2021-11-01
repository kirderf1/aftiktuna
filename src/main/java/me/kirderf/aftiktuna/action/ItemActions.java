package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.*;
import me.kirderf.aftiktuna.util.OptionalFunction;

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
		Optional<GameObject> optionalItem = aftik.findNearest(OptionalFunction.of(GameObject::isItem).filter(type::matching));
		if(optionalItem.isPresent()) {
			
			GameObject item = optionalItem.get();
			Entity.MoveResult move = aftik.tryMoveTo(item.getPosition());
			if (move.success()) {
				item.remove();
				aftik.addItem(type);
				
				System.out.printf("You picked up the %s.%n", type.lowerCaseName());
				return 1;
			} else {
				ActionHandler.printMoveFailure(move);
				return 0;
			}
		} else {
			System.out.printf("There is no %s here to pick up.%n", type.lowerCaseName());
			return 0;
		}
	}
	
	private static int wieldItem(GameInstance game, WeaponType itemType) {
		Aftik aftik = game.getAftik();
		
		if (aftik.wieldFromInventory(itemType)) {
			System.out.printf("You wielded a %s.%n", itemType.lowerCaseName());
			return 1;
		} else {
			Optional<GameObject> optionalItem = aftik.findNearest(OptionalFunction.of(GameObject::isItem).filter(itemType::matching));
			if(optionalItem.isPresent()) {
				
				GameObject item = optionalItem.get();
				Entity.MoveResult move = aftik.tryMoveTo(item.getPosition());
				if (move.success()) {
					item.remove();
					aftik.wield(itemType);
					
					System.out.printf("You picked up and wielded the %s.%n", itemType.lowerCaseName());
					return 1;
				} else {
					ActionHandler.printMoveFailure(move);
					return 0;
				}
			} else {
				System.out.printf("There is no %s that you can wield.%n", itemType.lowerCaseName());
				return 0;
			}
		}
	}
}
