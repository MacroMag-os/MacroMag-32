uzytkownicy = {"admin": "1234"}
klasy = []
nauczyciele = []
uczniowie = []
plan_lekcji = {"Poniedziałek": [], "Wtorek": [], "Środa": [], "Czwartek": [], "Piątek": []}

def logowanie():
    print("=== LOGOWANIE DO DZIENNIKA ===")
    login = input("Login: ")
    haslo = input("Hasło: ")
    if uzytkownicy.get(login) == haslo:
        print("Zalogowano pomyślnie!\n")
        return True
    print("Błędny login lub hasło!")
    return False

def menu():
    while True:
        print("\n--- DZIENNIK ELEKTRONICZNY ---")
        print("1. Dodaj klasę")
        print("2. Dodaj nauczyciela")
        print("3. Dodaj ucznia")
        print("4. Edytuj plan lekcji")
        print("5. Wyświetl dane i plan")
        print("0. Wyjdź")
        wybor = input("Wybierz opcję: ")

        if wybor == "1":
            klasy.append(input("Nazwa klasy (np. 3A): "))
        elif wybor == "2":
            nauczyciele.append(input("Imię i nazwisko nauczyciela: "))
        elif wybor == "3":
            uczniowie.append(input("Imię, nazwisko i klasa ucznia: "))
        elif wybor == "4":
            dzien = input("Podaj dzień tygodnia (Poniedziałek-Piątek): ")
            if dzien in plan_lekcji:
                przedmiot = input("Podaj lekcję (np. 1. Matematyka): ")
                plan_lekcji[dzien].append(przedmiot)
            else:
                print("Niepoprawny dzień.")
        elif wybor == "5":
            print("Klasy:", klasy)
            print("Nauczyciele:", nauczyciele)
            print("Uczniowie:", uczniowie)
            print("Plan lekcji:", plan_lekcji)
        elif wybor == "0":
            break

if logowanie():
    menu()
