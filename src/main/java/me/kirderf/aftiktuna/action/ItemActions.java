package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.*;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;

import java.util.Optional;
import java.util.function.Consumer;

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
		return searchForAndIfNotBlocked(game, aftik, type,
				item -> takeItem(game, aftik, item),
				() -> game.out().printf("There is no %s here to pick up.%n", type.lowerCaseName()));
	}
	
	private static void takeItem(GameInstance game, Aftik aftik, Item item) {
		Optional<Entity.MoveFailure> result = aftik.moveAndTake(item);
		
		result.ifPresentOrElse(
				failure -> ActionHandler.printMoveFailure(game, failure),
				() -> game.out().printf("%s picked up the %s.%n", aftik.getDisplayName(), item.getType().lowerCaseName()));
	}
	
	private static int wieldItem(GameInstance game, WeaponType weaponType) {
		Aftik aftik = game.getAftik();
		
		if (aftik.wieldFromInventory(weaponType)) {
			game.out().printf("%s wielded a %s.%n", aftik.getDisplayName(), weaponType.lowerCaseName());
			return 1;
		} else {
			return searchForAndIfNotBlocked(game, aftik, weaponType,
					item -> wieldItem(game, aftik, item, weaponType),
					() -> game.out().printf("There is no %s that %s can wield.%n", weaponType.lowerCaseName(), aftik.getDisplayName()));
		}
	}
	
	private static void wieldItem(GameInstance game, Aftik aftik, Item item, WeaponType type) {
		Optional<Entity.MoveFailure> result = aftik.moveAndWield(item, type);
		
		result.ifPresentOrElse(
				failure -> ActionHandler.printMoveFailure(game,failure),
				() -> game.out().printf("%s picked up and wielded the %s.%n", aftik.getDisplayName(), type.lowerCaseName()));
	}
	
	private static int searchForAndIfNotBlocked(GameInstance game, Aftik aftik, ObjectType type, Consumer<Item> onSuccess, Runnable onNoMatch) {
		Optional<Item> optionalItem = aftik.findNearest(Item.CAST.filter(type::matching));
		if (optionalItem.isPresent()) {
			Item item = optionalItem.get();
			
			Optional<GameObject> blocking = aftik.findBlockingTo(item.getCoord());
			if (blocking.isEmpty()) {
				onSuccess.accept(item);
				return 1;
			} else {
				ActionHandler.printBlocking(game, blocking.get());
				return 0;
			}
		} else {
			onNoMatch.run();
			return 0;
		}
	}
}