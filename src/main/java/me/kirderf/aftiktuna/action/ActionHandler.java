package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Creature;
import me.kirderf.aftiktuna.level.object.entity.Entity;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;
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
			game.out().printf("Unexpected input \"%s\"%n", input);
			return 0;
		}
	}
	
	private static int attack(GameInstance game, ObjectType creatureType) {
		Aftik aftik = game.getAftik();
		
		Optional<Creature> optionalCreature = aftik.findNearest(OptionalFunction.of(creatureType::matching).flatMap(Creature.CAST));
		if (optionalCreature.isPresent()) {
			Creature creature = optionalCreature.get();
			
			Entity.MoveAndAttackResult result = aftik.moveAndAttack(creature);
			
			result.either().run(attack -> printAttackAction(game, aftik, attack), move -> printMoveFailure(game, move));
			
			return result.success() ? 1 : 0;
		} else {
			game.out().println("There is no such creature to attack.");
			return 0;
		}
	}
	
	private static int launchShip(GameInstance game) {
		Aftik aftik = game.getAftik();
		
		if (aftik.getRoom() == game.getShip().getRoom()) {
			boolean result = game.tryLaunchShip(aftik);
			
			if (result) {
				game.out().printf("%s got fuel to the ship.%n", aftik.getName());
			} else {
				game.out().println("The ship can't be launched at this time.");
			}
			
			return 1;
		} else {
			game.out().printf("%s need to be in the ship in order to launch it.%n", aftik.getName());
			return 0;
		}
	}
	
	private static int controlAftik(GameInstance game, String name) {
		Optional<Aftik> aftikOptional = game.findByName(name);
		if (aftikOptional.isPresent()) {
			Aftik aftik = aftikOptional.get();
			if (aftik != game.getAftik()) {
				game.setControllingAftik(aftik);
				game.printStatus();
			} else {
				game.out().println("You're already in control of them.");
			}
		} else {
			game.out().println("There is no crew member by that name.");
		}
		return 0;
	}
	
	static void printMoveFailure(GameInstance game, Entity.MoveFailure result) {
		printBlocking(game, result.blocking());
	}
	
	static void printBlocking(GameInstance game, GameObject blocking) {
		game.out().printf("The %s is blocking the way.%n", blocking.getType().name());
	}
	
	private static void printAttackAction(GameInstance game, Entity attacker, AttackResult result) {
		Entity attacked = result.attacked();
		switch(result.type()) {
			case DIRECT_HIT -> game.out().printf(condition("%s got a direct hit on[ and killed] %s.%n", result.isKill()),
					attacker.getDisplayName(true, true), attacked.getDisplayName(true, false));
			case GRAZING_HIT -> game.out().printf(condition("%s's attack grazed[ and killed] %s.%n", result.isKill()),
					attacker.getDisplayName(true, true), attacked.getDisplayName(true, false));
			case DODGE -> game.out().printf("%s dodged %s's attack.%n",
					attacked.getDisplayName(true, true), attacker.getDisplayName(true, false));
		}
	}
	
	public void handleEntities(GameInstance game) {
		
		for (Aftik aftik : game.getGameObjectStream().flatMap(Aftik.CAST.toStream()).collect(Collectors.toList())) {
			if (aftik.isAlive() && aftik != game.getAftik()) {
				aftik.performAction(new ContextPrinter(game));
			}
		}
		
		for (Creature creature : game.getGameObjectStream().flatMap(Creature.CAST.toStream()).collect(Collectors.toList())) {
			handleCreature(game, creature);
		}
	}
	
	private static void handleCreature(GameInstance game, Creature creature) {
		if (creature.isAlive()) {
			Optional<AttackResult> result = creature.doAction();
			result.ifPresent(attack -> printAttackAction(game, creature, attack));
		}
	}
	
	private static String condition(String text, boolean b) {
		if (b)
			return text.replaceAll("[\\[\\]]", "");
		else return text.replaceAll("\\[.*]", "");
	}
}
