package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.InputReader;
import me.kirderf.aftiktuna.Main;
import me.kirderf.aftiktuna.location.Location;

import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

public class LocationSelector {
	private final List<LocationCategory> remainingCategories = Locations.categories.stream()
			.map(LocationCategory::mutableCopy).collect(Collectors.toCollection(ArrayList::new));
	
	public Location getRandomLocation(PrintWriter out, InputReader in) {
		LocationCategory category = pickCategory(out, in);
		Location location = category.createAndRemoveRandom(GameInstance.RANDOM);
		if (category.isEmpty())
			remainingCategories.remove(category);
		return location;
	}
	
	private LocationCategory pickCategory(PrintWriter out, InputReader in) {
		if (remainingCategories.size() == 1)
			return remainingCategories.get(0);
		
		int i1 = GameInstance.RANDOM.nextInt(remainingCategories.size());
		LocationCategory category1 = remainingCategories.get(i1);
		int i2 = GameInstance.RANDOM.nextInt(remainingCategories.size() - 1);
		if (i1 <= i2)
			i2++;
		LocationCategory category2 = remainingCategories.get(i2);
		
		out.println("-".repeat(Main.EXPECTED_LINE_LENGTH));
		out.printf("There are two destination targets: %s, %s%n", category1.getName(), category2.getName());
		out.printf("Pick the location to travel to next.%n%n");
		
		while (true) {
			String input = in.readLine();
			if (input.equalsIgnoreCase(category1.getName()))
				return category1;
			else if (input.equalsIgnoreCase(category2.getName()))
				return category2;
			else
				out.printf("Unexpected input \"%s\"%n", input);
		}
	}
}