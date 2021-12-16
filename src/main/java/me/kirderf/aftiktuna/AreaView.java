package me.kirderf.aftiktuna;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.game.GameCommands;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.entity.Aftik;

import java.io.PrintWriter;
import java.util.*;

public final class AreaView extends GameView {
	private final GameInstance game;
	
	public AreaView(GameInstance game) {
		this.game = game;
	}
	
	@Override
	public int handleInput(String input, CommandContext context) throws CommandSyntaxException {
		return GameCommands.handleInput(context, input);
	}
	
	@Override
	public void printView(PrintWriter out) {
		Aftik aftik = game.getCrew().getAftik();
		Area area = aftik.getArea();
		
		Map<GameObject, Character> symbolTable = new HashMap<>();
		Map<Character, String> nameTable = new HashMap<>();
		
		buildTables(aftik, area, symbolTable, nameTable);
		
		printAreaMap(area, symbolTable, out);
		printObjectLabels(nameTable, out);
	}
	
	private static void buildTables(Aftik aftik, Area area, Map<GameObject, Character> symbolTable, Map<Character, String> nameTable) {
		
		char spareSymbol = '0';
		for (GameObject object : area.objectStream()
				.sorted(Comparator.comparing(GameObject::hasCustomName, Boolean::compareTo)).toList()) {	//Let objects without a custom name get chars first
			char symbol = object.getDisplaySymbol();
			String name = object.getViewLabel(aftik);
			if (nameTable.containsKey(symbol) && !name.equals(nameTable.get(symbol)))
				symbol = spareSymbol++;
			
			symbolTable.put(object, symbol);
			nameTable.put(symbol, name);
		}
	}
	
	private static void printAreaMap(Area area, Map<GameObject, Character> symbolTable, PrintWriter out) {
		List<List<GameObject>> objectsByPos = new ArrayList<>();
		for (int pos = 0; pos < area.getLength(); pos++)
			objectsByPos.add(new ArrayList<>());
		
		area.objectStream().forEach(object -> objectsByPos.get(object.getCoord()).add(object));
		
		for (List<GameObject> objectStack : objectsByPos)
			objectStack.sort(Comparator.comparingInt(GameObject::getWeight).reversed());
		
		int lines = Math.max(1, objectsByPos.stream().map(List::size).max(Integer::compare).orElse(0));
		
		out.printf("%s:%n", area.getLabel());
		for (int line = lines - 1; line >= 0; line--) {
			StringBuilder builder = new StringBuilder((line == 0 ? "_" : " ").repeat(area.getLength()));
			for (int pos = 0; pos < area.getLength(); pos++) {
				if (objectsByPos.get(pos).size() > line)
					builder.setCharAt(pos, symbolTable.get(objectsByPos.get(pos).get(line)));
			}
			out.println(builder);
		}
	}
	
	private static void printObjectLabels(Map<Character, String> nameTable, PrintWriter out) {
		StringBuilder builder = new StringBuilder();
		nameTable.forEach((symbol, name) -> {
			String label = "%s: %s".formatted(symbol, name);
			if (!builder.isEmpty()) {
				if (builder.length() + label.length() + 3 <= Main.EXPECTED_LINE_LENGTH)
					builder.append("   ");
				else {
					out.println(builder);
					builder.setLength(0);
				}
			}
			builder.append(label);
		});
		if (!builder.isEmpty())
			out.println(builder);
	}
}