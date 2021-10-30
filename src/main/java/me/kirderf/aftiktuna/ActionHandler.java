package me.kirderf.aftiktuna;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Locale;
import java.util.Optional;

public class ActionHandler {
	private static final CommandDispatcher<GameInstance> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(literal("take").then(argument("item", ObjectArgument.create(ObjectType.ITEMS))
				.executes(context -> takeItem(context.getSource(), ObjectArgument.getType(context, "item")))));
		DISPATCHER.register(literal("enter").then(argument("door", ObjectArgument.create(ObjectType.DOORS))
				.executes(context -> goThroughDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
		DISPATCHER.register(literal("force").then(argument("door", ObjectArgument.create(ObjectType.DOORS))
				.executes(context -> forceDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
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
	
	private static int takeItem(GameInstance game, ObjectType type) {
		Aftik aftik = game.getAftik();
		Optional<GameObject> optionalItem = aftik.findNearest(OptionalFunction.of(GameObject::isItem).filter(type::matching));
		if(optionalItem.isPresent()) {
			
			GameObject item = optionalItem.get();
			aftik.moveTo(item.getPosition());
			item.remove();
			aftik.addItem(type);
			
			System.out.printf("You picked up the %s.%n", type.name().toLowerCase(Locale.ROOT));
			return 1;
		} else {
			System.out.printf("There is no %s here to pick up.%n", type.name().toLowerCase(Locale.ROOT));
			return 0;
		}
	}
	
	private static int goThroughDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(GameObject::getAsDoor));
		if(optionalDoor.isPresent()) {
			
			aftik.moveTo(optionalDoor.get().getPosition());
			optionalDoor.get().enter(aftik);
			return 1;
		} else {
			System.out.println("There is no such door here to go through.");
			return 0;
		}
	}
	
	private static int forceDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(GameObject::getAsDoor));
		if(optionalDoor.isPresent()) {
			
			aftik.moveTo(optionalDoor.get().getPosition());
			optionalDoor.get().force(aftik);
			return 1;
		} else {
			System.out.println("There is no such door here.");
			return 0;
		}
	}
}
