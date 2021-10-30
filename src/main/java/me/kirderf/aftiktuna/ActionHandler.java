package me.kirderf.aftiktuna;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.Door;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Optional;

public class ActionHandler {
	private static final CommandDispatcher<GameInstance> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(literal("take").then(literal("fuel").then(literal("can").executes(context -> takeFuelCan(context.getSource())))));
		DISPATCHER.register(literal("go").then(literal("through").then(argument("door", ObjectArgument.create(ObjectType.DOORS))
				.executes(context -> goThroughDoor(context.getSource(), ObjectArgument.getType(context, "door"))))));
	}
	
	private static LiteralArgumentBuilder<GameInstance> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	private static <T> RequiredArgumentBuilder<GameInstance, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public static int handleInput(GameInstance game, String input) {
		try {
			return DISPATCHER.execute(input, game);
		} catch(CommandSyntaxException ignored) {
			System.out.printf("Unexpected input \"%s\"%n", input);
			return 0;
		}
	}
	
	private static int takeFuelCan(GameInstance game) {
		GameObject aftik = game.getAftik();
		Optional<GameObject> optionalFuel = aftik.findNearest(GameObject::isFuelCan);
		if (optionalFuel.isPresent()) {
			
			aftik.moveTo(optionalFuel.get().getPosition());
			optionalFuel.get().remove();
			System.out.println("You picked up the fuel can.");
			
			game.setHasWon();
		} else {
			System.out.println("There is no fuel can here to pick up.");
		}
		return 1;
	}
	
	private static int goThroughDoor(GameInstance game, ObjectType doorType) {
		GameObject aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(GameObject::getAsDoor));
		if (optionalDoor.isPresent()) {
			
			optionalDoor.get().enter(aftik);
			System.out.println("You entered the door into a new room.");
		} else {
			System.out.println("There is no such door here to go through.");
		}
		return 1;
	}
}
