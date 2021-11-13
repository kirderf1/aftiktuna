package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.action.result.AttackResult;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Ship;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Creature;
import me.kirderf.aftiktuna.level.object.entity.Entity;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;
import java.util.function.Consumer;
import java.util.stream.Collectors;

public final class ActionHandler {
	private final CommandDispatcher<GameInstance> dispatcher = new CommandDispatcher<>();
	
	public ActionHandler() {
		ItemActions.register(dispatcher);
		DoorActions.register(dispatcher);
		dispatcher.register(literal("attack").then(argument("creature", ObjectArgument.create(ObjectTypes.CREATURES))
				.executes(context -> attack(context.getSource(), ObjectArgument.getType(context, "creature")))));
		dispatcher.register(literal("launch").then(literal("ship").executes(context -> launchShip(context.getSource()))));
		dispatcher.register(literal("control").then(argument("name", StringArgumentType.string())
				.executes(context -> controlAftik(context.getSource(), StringArgumentType.getString(context, "name")))));
		dispatcher.register(literal("wait").executes(context -> 1));
	}
	
	static LiteralArgumentBuilder<GameInstance> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	static <T> RequiredArgumentBuilder<GameInstance, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public int handleInput(GameInstance game, String input) {
		try {
			return dispatcher.execute(input, game);
		} catch(CommandSyntaxException ignored) {
			game.directOut().printf("Unexpected input \"%s\"%n", input);
			return 0;
		}
	}
	
	private static int attack(GameInstance game, ObjectType creatureType) {
		Aftik aftik = game.getAftik();
		
		Optional<Creature> optionalCreature = aftik.findNearest(OptionalFunction.of(creatureType::matching).flatMap(Creature.CAST));
		if (optionalCreature.isPresent()) {
			Creature creature = optionalCreature.get();
			
			aftik.moveAndAttack(creature, game.out());
			
			return 1;
		} else {
			game.directOut().println("There is no such creature to attack.");
			return 0;
		}
	}
	
	private static int launchShip(GameInstance game) {
		Aftik aftik = game.getAftik();
		
		if (aftik.hasItem(ObjectTypes.FUEL_CAN)) {
			if (isNearShip(aftik, game.getCrew().getShip())) {
				aftik.getMind().setLaunchShip(game.out());
				
				return 1;
			} else {
				game.directOut().printf("%s need to be near the ship in order to launch it.%n", aftik.getName());
				return 0;
			}
		} else {
			game.directOut().printf("%s need a fuel can to launch the ship.%n", aftik.getName());
			return 0;
		}
	}
	
	private static boolean isNearShip(Aftik aftik, Ship ship) {
		return aftik.getRoom() == ship.getRoom() || aftik.findNearest(Door.CAST.filter(ObjectTypes.SHIP_ENTRANCE::matching)).isPresent();
	}
	
	private static int controlAftik(GameInstance game, String name) {
		Optional<Aftik> aftikOptional = game.getCrew().findByName(name);
		if (aftikOptional.isPresent()) {
			Aftik aftik = aftikOptional.get();
			if (aftik != game.getAftik()) {
				game.setControllingAftik(aftik);
				game.printStatus();
			} else {
				game.directOut().println("You're already in control of them.");
			}
		} else {
			game.directOut().println("There is no crew member by that name.");
		}
		return 0;
	}
	
	static <T extends GameObject> int searchForAndIfNotBlocked(GameInstance game, Aftik aftik, OptionalFunction<GameObject, T> mapper, Consumer<T> onSuccess, Runnable onNoMatch) {
		Optional<T> optionalDoor = aftik.findNearest(mapper);
		if (optionalDoor.isPresent()) {
			T object = optionalDoor.get();
			
			Optional<GameObject> blocking = aftik.findBlockingTo(object.getCoord());
			if (blocking.isEmpty()) {
				onSuccess.accept(object);
				return 1;
			} else {
				ActionHandler.printBlocking(game.out(), aftik, blocking.get());
				return 0;
			}
		} else {
			onNoMatch.run();
			return 0;
		}
	}
	
	public static void printMoveFailure(ContextPrinter out, Entity entity, Entity.MoveFailure result) {
		printBlocking(out, entity, result.blocking());
	}
	
	static void printBlocking(ContextPrinter out, Entity entity, GameObject blocking) {
		out.printFor(entity, "The %s is blocking the way.%n", blocking.getType().name());
	}
	
	public static void printAttackAction(ContextPrinter out, Entity attacker, AttackResult result) {
		Entity attacked = result.attacked();
		switch(result.type()) {
			case DIRECT_HIT -> out.printAt(attacker, condition("%s got a direct hit on[ and killed] %s.%n", result.isKill()),
					attacker.getDisplayName(true, true), attacked.getDisplayName(true, false));
			case GRAZING_HIT -> out.printAt(attacker, condition("%s's attack grazed[ and killed] %s.%n", result.isKill()),
					attacker.getDisplayName(true, true), attacked.getDisplayName(true, false));
			case DODGE -> out.printAt(attacker, "%s dodged %s's attack.%n",
					attacked.getDisplayName(true, true), attacker.getDisplayName(true, false));
		}
	}
	
	public void handleEntities(GameInstance game) {
		
		for (Entity entity : game.getGameObjectStream().flatMap(Entity.CAST.toStream()).collect(Collectors.toList())) {
			if (entity.isAlive() && entity != game.getAftik()) {
				entity.performAction(game.out());
			}
		}
	}
	
	private static String condition(String text, boolean b) {
		if (b)
			return text.replaceAll("[\\[\\]]", "");
		else return text.replaceAll("\\[.*]", "");
	}
}
