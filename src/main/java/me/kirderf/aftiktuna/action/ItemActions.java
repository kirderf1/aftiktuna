package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.WeaponType;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;
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
			Optional<Entity.MoveFailure> move = aftik.tryMoveTo(item.getPosition());
			if (move.isEmpty()) {
				item.remove();
				aftik.addItem(type);
				
				game.out().printf("You picked up the %s.%n", type.lowerCaseName());
				return 1;
			} else {
				ActionHandler.printMoveFailure(game, move.get());
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
			Optional<GameObject> optionalItem = aftik.findNearest(OptionalFunction.of(GameObject::isItem).filter(itemType::matching));
			if(optionalItem.isPresent()) {
				
				GameObject item = optionalItem.get();
				Optional<Entity.MoveFailure> move = aftik.tryMoveTo(item.getPosition());
				if (move.isEmpty()) {
					item.remove();
					aftik.wield(itemType);
					
					game.out().printf("You picked up and wielded the %s.%n", itemType.lowerCaseName());
					return 1;
				} else {
					ActionHandler.printMoveFailure(game, move.get());
					return 0;
				}
			} else {
				game.out().printf("There is no %s that you can wield.%n", itemType.lowerCaseName());
				return 0;
			}
		}
	}
}
