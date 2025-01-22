# **Cron Twit**

Cron Twit is a lightweight and efficient Rust-based application designed to automate tweet scheduling. Users can plan their weekly tweets through an intuitive UI, specify posting times, and manage their Twitter activity effortlessly.

---

## **Features**

### **Weekly Tweet Scheduling**
- Users can input the number of tweets they want to post in a week.  
- Provide the tweet content along with the desired posting times.

### **UI for Ease of Use**
The application includes a user-friendly interface where users can:  
- Add tweet content for the week.  
- Specify the number of tweets to be posted.  
- Set the exact time for each tweet.  

### **Weekly Reminder**
Before the start of the next week, the system sends a reminder notification to the user to schedule their tweets for the upcoming week.

### **Twitter Integration**
Tweets are automatically posted to Twitter using the Twitter API.

### **Rust Performance**
The application leverages Rust’s concurrency and efficiency to ensure reliable and performant scheduling.

---

## **Setup Instructions**

### **Prerequisites**
1. Rust (latest stable version).  
2. A registered Twitter developer account and API keys.  
3. Docker (optional, for containerized deployment).  

### **Installation**

1. Clone the repository:  
   ```bash
   git clone https://github.com/bulutcan99/twit-cron
   cd twit-cron
   ```  

2. Install dependencies:  
   ```bash
   cargo build --release
   ```  

3. Set up environment variables:  
   Create a `.env` file in the project root and include the following:  
   ```env
   TWITTER_API_KEY=<your_api_key>
   TWITTER_API_SECRET=<your_api_secret>
   TWITTER_ACCESS_TOKEN=<your_access_token>
   TWITTER_ACCESS_SECRET=<your_access_secret>
   ```  

4. Run the application:  
   ```bash
   cargo run
   ```  

---

## **Docker Deployment**

1. Build and run the containerized version of Cron Twit:  
   ```bash
   docker build -t cron-twit .  
   docker run -d --env-file .env cron-twit  
   ```  

---

## **How It Works**

### **1. Schedule Tweets**
The user interface allows users to:  
- Enter tweet content.  
- Set the number of tweets to post per week.  
- Define the exact time for each tweet.  

### **2. Weekly Workflow**
- Tweets are queued and posted at the specified times during the week.  
- A reminder notification is sent to users before the new week starts, prompting them to schedule tweets for the upcoming week.  

### **3. Tweet Posting**
The app uses the Twitter API to post tweets reliably at the scheduled times.  

---

## **License**

This project is licensed under the **Apache-2.0 License**. See the `LICENSE` file for details.
