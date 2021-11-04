package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.AttackResult;
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
	
	public void handleCreatures(GameInstance game) {
		game.getGameObjectStream().flatMap(Creature.CAST.toStream()).collect(Collectors.toList()).forEach(creature -> handleCreature(game, creature));
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
