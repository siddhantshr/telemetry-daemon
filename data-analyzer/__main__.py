import sqlite3
import matplotlib.pyplot as plt
import pandas as pd
import logging
import numpy as np

class DataAnalyzer:
    def __init__(self, db_path):
        self.db_path = db_path
        self.conn = sqlite3.connect(self.db_path)
        self.df_sys = None
        self.df_proc = None
        self.fig, self.axs = plt.subplots(2, 2, figsize=(15, 10))

    def fetch_data(self):
        self.df_sys = pd.read_sql_query("SELECT * FROM system_metrics", self.conn)
        self.df_proc = pd.read_sql_query("SELECT * FROM telemetry", self.conn)

    def report_system_anomalies(self):
        anomalies = self.df_sys[
            (self.df_sys['cpu_usage'] > 90) | (self.df_sys['memory_usage'] > 90)
        ]
        if not anomalies.empty:
            logging.warning("Anomalies detected in system metrics:")
            for _, row in anomalies.iterrows():
                logging.warning(f"Timestamp: {row['timestamp']}, CPU Usage: {row['cpu_usage']}%, Memory Usage: {row['memory_usage']}%")

    def report_process_anomalies(self):
        anomalies = self.df_proc[
            (self.df_proc['cpu_usage'] > 60) | (self.df_proc['memory_usage'] > 60)
        ]
        if not anomalies.empty:
            logging.warning("Anomalies detected in process metrics:")
            for _, row in anomalies.iterrows():
                logging.warning(
                    f"HIGH SYSTEM RESOURCE USAGE - Timestamp: {row['timestamp']}, Process: {row['command']}, CPU Usage: {row['cpu_usage']}%, Memory Usage: {row['memory_usage']}%"
                )
    def plot_proc_metrics(self):
        avg_usage = pd.read_sql_query(
            "SELECT command, AVG(cpu_usage) as avg_cpu, AVG(memory_usage) as avg_mem FROM telemetry GROUP BY command ORDER BY avg_cpu DESC",
            self.conn
        )

        for index, row in avg_usage.iterrows():
            logging.info(f"Process: {row['command']}, Avg CPU Usage: {row['avg_cpu']}%, Avg Memory Usage: {row['avg_mem']}%")

        avg_usage = avg_usage[(avg_usage['avg_cpu'] > 10) | (avg_usage['avg_mem'] > 10)]

        avg_usage['avg_cpu'] = avg_usage['avg_cpu'].clip(upper=100)
        avg_usage['avg_mem'] = avg_usage['avg_mem'].clip(upper=100)

        x = np.arange(len(avg_usage['command']))
        width = 0.4

        self.axs[0, 1].bar(
            x - width / 2,
            avg_usage['avg_cpu'],
            width=width,
            label='Avg CPU Usage',
            color='blue'
        )

        self.axs[0, 1].bar(
            x + width / 2,
            avg_usage['avg_mem'],
            width=width,
            label='Avg Memory Usage',
            color='orange',
        )

        self.axs[0, 1].set_xticks(x)
        self.axs[0, 1].set_xticklabels(avg_usage['command'], rotation=45)

        self.axs[0, 1].set_xlabel('Process Command')
        self.axs[0, 1].set_ylabel('Average Usage (%)')
        self.axs[0, 1].set_title('Average CPU and Memory Usage by Process')
        self.axs[0, 1].legend()
            

    def plot_system_metrics(self):
        self.df_sys['timestamp'] = pd.to_datetime(self.df_sys['timestamp'], unit="s")
        self.axs[0, 0].plot(self.df_sys['timestamp'], self.df_sys['cpu_usage'], label='CPU Usage')
        self.axs[0, 0].plot(self.df_sys['timestamp'], self.df_sys['memory_usage'], label='Memory Usage', color='orange')
        self.axs[0, 0].set_xlabel('Time')
        self.axs[0, 0].set_ylabel('CPU Usage (%)')
        self.axs[0, 0].set_title('CPU Usage Over Time')
        self.axs[0, 0].legend()

    def process_heatmap(self):
        self.df_proc["timestamp"] = pd.to_datetime(self.df_proc["timestamp"], unit="s")
        top_processes = (
            self.df_proc.groupby("command")["cpu_usage"]
            .mean()
            .sort_values(ascending=False)
            .head(10)
            .index
        )

        self.df_proc = self.df_proc[self.df_proc["command"].isin(top_processes)]
        heatmap_data = self.df_proc.pivot_table(
            index="command",
            columns="timestamp",
            values="cpu_usage",
            aggfunc="mean"
        )
        self.axs[1, 0].imshow(
            heatmap_data,
            aspect='auto',
            interpolation='nearest'
        )

        sc = self.axs[1, 0].imshow(
            heatmap_data,
            aspect='auto',
            interpolation='nearest',
            cmap='Reds',
            vmin=0,
            vmax=100
        )
        self.fig.colorbar(sc, label="CPU Usage %")

        self.axs[1, 0].set_yticks(range(len(heatmap_data.index)))
        self.axs[1, 0].set_yticklabels(heatmap_data.index)
        self.axs[1, 0].set_xticks(range(len(heatmap_data.columns)))
        self.axs[1, 0].set_xticklabels(
            [t.strftime('%H:%M:%S') for t in heatmap_data.columns],
            rotation=45
        )
        self.axs[1, 0].set_title("Process CPU Usage Heatmap")

    def report_recurring_processes(self):
        recurring = pd.read_sql_query(
            "SELECT command, COUNT(*) as count FROM telemetry GROUP BY command HAVING count > 5 ORDER BY count DESC",
            self.conn
        )
        if not recurring.empty:
            logging.info("Recurring processes detected:")
            for _, row in recurring.iterrows():
                logging.info(f"Process: {row['command']}, Count: {row['count']}")

    def report_stability(self):
        if self.df_proc is None or self.df_proc.empty:
            return

        stdevs = (
            self.df_proc
            .groupby("command", as_index=False)[["cpu_usage", "memory_usage"]]
            .std(ddof=0)
            .rename(columns={"cpu_usage": "cpu_stdev", "memory_usage": "mem_stdev"})
            .sort_values(by="cpu_stdev", ascending=False)
        )

        if not stdevs.empty:
            logging.info("Process stability report:")
            for _, row in stdevs.iterrows():
                if row["cpu_stdev"] > 20 or row["mem_stdev"] > 20:
                    logging.warning(
                        f"Process: {row['command']} has high variability - CPU Stdev: {row['cpu_stdev']:.2f}%, Memory Stdev: {row['mem_stdev']:.2f}%"
                    )
                else:
                    logging.info(
                        f"Process: {row['command']}, CPU Usage Stdev: {row['cpu_stdev']:.2f}%, Memory Usage Stdev: {row['mem_stdev']:.2f}%"
                    )

    def plot_cpu_vs_mem(self):
        df = self.df_proc[['cpu_usage', 'memory_usage']].clip(upper=100)
        distance = np.sqrt(df['cpu_usage']**2 + df['memory_usage']**2)

        scatter = self.axs[1,1].scatter(df['cpu_usage'], df['memory_usage'], alpha=0.5, c=distance, cmap='Reds')
        self.axs[1,1].set_xlabel('CPU Usage (%)')
        self.axs[1,1].set_ylabel('Memory Usage (%)')
        self.axs[1,1].set_title('CPU Usage vs Memory Usage for Processes')

        self.fig.colorbar(
            scatter,
            ax=self.axs[1,1],
            label='Distance from Origin (High Resource Usage)'
        )

def main():
    try:
        import os
        os.makedirs('./data-analyzer/out', exist_ok=True)
    except Exception as e:
        print(f"Failed to create output directory: {e}")
        return
    logging.basicConfig(
        level=logging.INFO,
        filename='./data-analyzer/out/analyzer.log',
        format='%(asctime)s - %(levelname)s - %(message)s'
    )
    logging.info("Starting Data Analyzer")

    try:
        Analyzer = DataAnalyzer("/var/lib/teld/telemetry.db")
    except Exception as e:
        logging.error(f"Failed to connect to database: {e}")
        return
    Analyzer.fetch_data()
    Analyzer.plot_system_metrics()
    Analyzer.report_system_anomalies()
    Analyzer.plot_proc_metrics()
    Analyzer.report_process_anomalies()
    Analyzer.process_heatmap()
    Analyzer.report_recurring_processes()
    Analyzer.report_stability()
    Analyzer.plot_cpu_vs_mem()
    plt.tight_layout()
    plt.savefig('./data-analyzer/out/analysis.png')
    plt.show()
    logging.info("Data analysis completed and saved to ./data-analyzer/out/analysis.png")
    print(
        "Analyzer report in ./data-analyzer/out/ directory"
    )


if __name__ == "__main__":
    main()