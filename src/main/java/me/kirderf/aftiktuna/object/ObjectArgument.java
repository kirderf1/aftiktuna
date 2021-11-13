package me.kirderf.aftiktuna.object;

import com.mojang.brigadier.StringReader;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.brigadier.exceptions.SimpleCommandExceptionType;
import com.mojang.brigadier.suggestion.Suggestions;
import com.mojang.brigadier.suggestion.SuggestionsBuilder;

import java.util.Collection;
import java.util.Locale;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

public class ObjectArgument<T extends ObjectType> implements ArgumentType<T> {
	private static final SimpleCommandExceptionType INVALID_TYPE = new SimpleCommandExceptionType(() -> "No such object type");
	
	private final Collection<T> types;
	
	private ObjectArgument(Collection<T> types) {
		this.types = types;
	}
	
	public static <T extends ObjectType> ObjectArgument<T> create(Collection<T> types) {
		return new ObjectArgument<>(types);
	}
	
	public static ObjectType getType(CommandContext<?> context, String name) {
		return getType(context, name, ObjectType.class);
	}
	
	public static <T extends ObjectType> T getType(CommandContext<?> context, String name, Class<T> clazz) {
		return context.getArgument(name, clazz);
	}
	
	@Override
	public T parse(StringReader reader) throws CommandSyntaxException {
		int start = reader.getCursor();
		String remaining = reader.getRemaining().toLowerCase(Locale.ROOT);
		for (T type : types) {
			String name = type.name().toLowerCase(Locale.ROOT);
			if (remaining.startsWith(name)) {
				reader.setCursor(start + name.length());
				return type;
			}
		}
		throw INVALID_TYPE.createWithContext(reader);
	}
	
	@Override
	public <S> CompletableFuture<Suggestions> listSuggestions(CommandContext<S> context, SuggestionsBuilder builder) {
		String remainder = builder.getRemainingLowerCase();
		for (ObjectType type : types) {
			String name = type.name().toLowerCase(Locale.ROOT);
			if (name.startsWith(remainder))
				builder.suggest(name);
		}
		return builder.buildFuture();
	}
	
	@Override
	public Collection<String> getExamples() {
		return types.stream().map(ObjectType::name).collect(Collectors.toList());
	}
}