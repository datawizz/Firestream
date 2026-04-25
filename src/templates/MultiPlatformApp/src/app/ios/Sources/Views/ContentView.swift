import SwiftUI

struct ContentView: View {
    @State private var count = 0

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Text("Multi-Platform App")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text("iOS")
                    .font(.title2)
                    .foregroundStyle(.secondary)

                Divider()

                VStack(spacing: 12) {
                    Text("Counter: \(count)")
                        .font(.title3)
                        .monospacedDigit()

                    HStack(spacing: 12) {
                        Button("Decrement") { count -= 1 }
                            .buttonStyle(.bordered)

                        Button("Reset") { count = 0 }
                            .buttonStyle(.bordered)

                        Button("Increment") { count += 1 }
                            .buttonStyle(.borderedProminent)
                    }
                }

                Spacer()
            }
            .padding()
            .navigationTitle("Home")
        }
    }
}

#Preview {
    ContentView()
}
